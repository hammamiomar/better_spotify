use std::{env, sync::{Arc, RwLock}};
use anyhow::Result;
use axum::{response::Redirect, routing::get, extract::State as AxumState};
use dioxus::prelude::*;
use dioxus_logger;
use dotenvy::dotenv;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use tower_sessions::{Expiry, MemoryStore, Session, SessionManagerLayer};
use time::Duration;

use crate::{api_models::SpotifyTokenResponse, App, db::Database};
use crate::auth::pkce;


#[derive(Clone)]
pub struct AppState {
    pub database: Database,
    // Keep the old in-memory tokens for backward compatibility
    pub current_user_tokens: Arc<RwLock<Option<SpotifyTokenResponse>>>,
}

impl AppState {
    pub async fn new() -> Result<Self> {
        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "sqlite:data.db".to_string());
        
        let database = Database::new(&database_url).await?;
        
        Ok(Self { 
            database,
            current_user_tokens: Arc::new(RwLock::new(None)),
        })
    }
}

#[cfg(feature = "server")]
pub async fn start_server() -> Result<()> {
    use std::any::Any;


    dotenv().ok();

    dioxus_logger::init(tracing::Level::INFO).expect("failed to init logger");

    let _client_id = env::var("SPOTIFY_CLIENT_ID")
        .expect("SPOTIFY_CLIENT_ID must be set in .env");

    let app_state = AppState::new().await?;

    let provider = {
        let shared = app_state.clone();
        move || Box::new(shared.clone()) as Box<dyn Any>
    };

    let cfg = ServeConfigBuilder::default()
        .context_providers(Arc::new(vec![Box::new(provider)]));

     let address = dioxus::cli_config::fullstack_address_or_localhost();

    // Set up session management
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false) // Set to true in production with HTTPS
        .with_expiry(Expiry::OnInactivity(Duration::seconds(3600))); // 1 hour

    let axum_router = axum::Router::new()
        .route("/login", get(spotify_login_handler))
        .route("/callback", get(spotify_callback_handler))
        .route("/logout", get(logout_handler))
        .serve_dioxus_application(cfg,App)
        .layer(session_layer)
        .with_state(app_state.clone());

    let listener = tokio::net::TcpListener::bind(address).await?;
    axum::serve(listener, axum_router.into_make_service())
        .await?;

    Ok(())
}

async fn spotify_login_handler(
    AxumState(app_state): AxumState<AppState>,
    _session: Session,
) -> Redirect {

    let client_id = env::var("SPOTIFY_CLIENT_ID")
        .expect("sptify client id must be set");
    let redirect_uri = env::var("SPOTIFY_REDIRECT_URI")
        .expect("Spotify redirect uri must be set");

    //generate pkce codes
    let code_verifier = pkce::generate_code_verifier();
    let code_challenge = pkce::generate_code_challenge(&code_verifier);

    // generate random state param for CSRF protection
    let state:String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(char::from)
        .collect();

    // Store code verifier in database
    if let Err(e) = app_state.database.store_pkce_verifier(&state, &code_verifier).await {
        tracing::error!("Failed to store PKCE verifier: {}", e);
        return Redirect::temporary("/login?error=internal_error");
    }
    
    tracing::info!("Stored verifier for state: {}", state);

    let scope = "playlist-read-private playlist-read-collaborative playlist-modify-private
    user-read-private user-read-email ugc-image-upload";

    // construct URL

    let mut auth_url = reqwest::Url::parse("https://accounts.spotify.com/authorize").unwrap();
    auth_url.query_pairs_mut()
        .append_pair("response_type","code")
        .append_pair("client_id", &client_id)
        .append_pair("scope", scope)
        .append_pair("redirect_uri", &redirect_uri)
        .append_pair("state", &state)
        .append_pair("code_challenge_method", "S256")
        .append_pair("code_challenge", &code_challenge);
    tracing::info!("Redirecting user to spotify: {}", auth_url);


    Redirect::temporary(auth_url.as_str())
}



async fn spotify_callback_handler(
    AxumState(app_state): AxumState<AppState>,
    session: Session,
    query: axum::extract::Query<std::collections::HashMap<String,String>>,
) -> Redirect {
    // query will either respond with  code and state, or error and state
    
    let code = match query.get("code"){
        Some(c) => c.clone(),
        None => {
            tracing::error!("Callback missing code param");
            return Redirect::temporary("/login?error=missing_code");
        }
    };

    let received_state = match query.get("state"){
        Some(s) => s.clone(),
        None => {
            tracing::error!("Callback missing state param");
            return Redirect::temporary("/login?error=missing_state");
        }
    };

    // Get and remove code verifier from database for CSRF protection
    let code_verifier = match app_state.database.get_and_remove_pkce_verifier(&received_state).await {
        Ok(Some(verifier)) => verifier,
        Ok(None) => {
            tracing::error!("State mismatch or verifier not found/expired for state: {}", received_state);
            return Redirect::temporary("/login?error=state_mismatch");
        }
        Err(e) => {
            tracing::error!("Database error retrieving PKCE verifier: {}", e);
            return Redirect::temporary("/login?error=internal_error");
        }
    };
    tracing::info!("Retrieved verifier for state: {}", received_state);

    let client_id = env::var("SPOTIFY_CLIENT_ID").expect("SPOTIFY_CLIENT_ID must be set");
    let redirect_uri = env::var("SPOTIFY_REDIRECT_URI").expect("SPOTIFY_REDIRECT_URI must be set");
    let client_secret = env::var("SPOTIFY_CLIENT_SECRET").expect("SPOTIFY_CLIENT_SECRET must be set");

    // prepare token request

    let client = reqwest::Client::new();
    let token_endpoint = "https://accounts.spotify.com/api/token";

    let params = [
        ("grant_type", "authorization_code"),
        ("code", &code),
        ("redirect_uri",&redirect_uri),
        ("client_id",&client_id),
        ("code_verifier",&code_verifier),
    ];

    let auth_header_value = format!(
        "Basic {}",
        URL_SAFE_NO_PAD.encode(format!("{}:{}", client_id,client_secret))
    );

    tracing::info!("Requesting Access Token");

    let response_result = client
        .post(token_endpoint)
        .header(AUTHORIZATION, auth_header_value)
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .header(ACCEPT, "application/json")
        .form(&params)
        .send()
        .await;

    match response_result{
        Ok(token_response) =>{
            if token_response.status().is_success(){
                match token_response.json::<SpotifyTokenResponse>().await{
                    Ok(token_response) => {
                        tracing::info!("Successfully obtained tokens");

                        // First, get the user's Spotify profile to get their ID
                        let client = reqwest::Client::new();
                        let profile_result = client
                            .get("https://api.spotify.com/v1/me")
                            .bearer_auth(&token_response.access_token)
                            .send()
                            .await;

                        match profile_result {
                            Ok(profile_response) => {
                                if profile_response.status().is_success() {
                                    match profile_response.json::<crate::api_models::SpotifyUserProfile>().await {
                                        Ok(profile) => {
                                            // Find or create user
                                            let user = match app_state.database.get_user_by_spotify_id(&profile.id).await {
                                                Ok(Some(existing_user)) => existing_user,
                                                Ok(None) => {
                                                    match app_state.database.create_user(&profile.id).await {
                                                        Ok(new_user) => new_user,
                                                        Err(e) => {
                                                            tracing::error!("Failed to create user: {}", e);
                                                            return Redirect::temporary("/login?error=user_creation_failed");
                                                        }
                                                    }
                                                }
                                                Err(e) => {
                                                    tracing::error!("Database error finding user: {}", e);
                                                    return Redirect::temporary("/login?error=database_error");
                                                }
                                            };

                                            // Store tokens in database AND in-memory for compatibility
                                            match app_state.database.store_user_token(&user.id, &token_response).await {
                                                Ok(_) => {
                                                    // Store user ID in session
                                                    if let Err(e) = session.insert("user_id", &user.id).await {
                                                        tracing::error!("Failed to store user ID in session: {}", e);
                                                    }
                                                    
                                                    // Also store in memory for backward compatibility
                                                    *app_state.current_user_tokens.write().unwrap() = Some(token_response);
                                                    
                                                    tracing::info!("Successfully authenticated user: {}", profile.id);
                                                    Redirect::temporary("/")
                                                }
                                                Err(e) => {
                                                    tracing::error!("Failed to store user tokens: {}", e);
                                                    Redirect::temporary("/login?error=token_storage_failed")
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            tracing::error!("Failed to parse user profile: {}", e);
                                            Redirect::temporary("/login?error=profile_parse_failed")
                                        }
                                    }
                                } else {
                                    tracing::error!("Failed to get user profile, status: {}", profile_response.status());
                                    Redirect::temporary("/login?error=profile_fetch_failed")
                                }
                            }
                            Err(e) => {
                                tracing::error!("Network error fetching user profile: {}", e);
                                Redirect::temporary("/login?error=network_error")
                            }
                        }
                    }

                    Err(e) => {
                        tracing::error!( "failed to parse token response json:{}",e);
                        Redirect::temporary("/login?error=token_parse_failed")
                    }
                }
            }else{
                let status = token_response.status();
                let text = token_response.text().await.unwrap_or_else(|_| {
                    "Failed to read error body".to_string()
                });
                tracing::error!("Token request failed with status {}:{}", status,text);
                Redirect::temporary("/login?error=token_request_failed")
            }
        }
        Err(e) =>{
            tracing::error!("Failed to send token request: {}", e);
            Redirect::temporary("/login?error=network_error")
        }
    }
}

async fn logout_handler(session: Session) -> Redirect {
    if let Err(e) = session.flush().await {
        tracing::error!("Failed to flush session during logout: {}", e);
    }
    tracing::info!("User logged out");
    Redirect::temporary("/")
}
