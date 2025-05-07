use std::env;
use anyhow::Result;
use axum::{response::Redirect, routing::get};
use dioxus::prelude::*;
use dioxus_logger;
use dotenvy::dotenv;
use std::{collections::HashMap, sync::Mutex};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use lazy_static::lazy_static;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use serde::Deserialize;
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};

use crate::App;
use crate::auth::pkce;

lazy_static!{
    static ref PKCE_VERIFIERS: Mutex<HashMap<String,String>> = Mutex::new(HashMap::new());
}

lazy_static!{
    static ref CURRENT_USER_TOKEN: Mutex<Option<SpotifyTokenResponse>> = Mutex::new(None);
}


pub async fn start_server() -> Result<()> {

    dotenv().ok();

    dioxus_logger::init(tracing::Level::INFO).expect("failed to init logger");

    let client_id = env::var("SPOTIFY_CLIENT_ID")
        .expect("SPOTIFY_CLIENT_ID must be set in .env");

    let address = dioxus::cli_config::fullstack_address_or_localhost();

    let axum_router = axum::Router::new()
        .route("/login", get(spotify_login_handler))
        .route("/callback", get(spotify_callback_handler))
        .serve_dioxus_application(ServeConfigBuilder::new(),App);

    let listener = tokio::net::TcpListener::bind(address).await?;
    axum::serve(listener, axum_router.into_make_service())
        .await?;

    Ok(())
}

async fn spotify_login_handler() -> Redirect{

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

    //stroing code verifier temporarily

    PKCE_VERIFIERS.lock().unwrap().insert(state.clone(), code_verifier);
    tracing::info!("Stored verifier for state:{}",state);

    let scope = "playlist-read-private playlist-read-collaborative 
    user-read-private user-read-email";

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

#[derive(Deserialize, Debug)]
struct SpotifyTokenResponse{
    access_token:String,
    token_type:String,
    scope:String,
    expires_in: u64,
    refresh_token : Option<String>,
}

async fn spotify_callback_handler(
    query: axum::extract::Query<std::collections::HashMap<String,String>>,
) -> Redirect{
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

    // get code verifier and match state, for CSRF protection

    let code_verifier = {
        let mut verifiers = PKCE_VERIFIERS.lock().unwrap();

        match verifiers.remove(&received_state){
            Some(v) => v,
            None => {
                tracing::error!("state mismatch OR verifier not found for state");
                return Redirect::temporary("/login?error=state_mismatch")
            }
        }
    };
    tracing::info!("Retrieved verifier for state: {}",received_state);

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
                    Ok(token_reponse) =>{
                        tracing::info!("Succesfully obtained tokens: {:?}", token_reponse);

                        //TODO Change AS its only for solo dev

                        *CURRENT_USER_TOKEN.lock().unwrap() = Some(token_reponse);

                        Redirect::temporary("/")
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
