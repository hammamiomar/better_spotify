use dioxus::prelude::*;
use reqwest::Client;
use crate::api_models::{SpotifyPlaylistsResponse, SpotifyTokenResponse, SpotifyUserProfile};

#[cfg(feature="server")]
use crate::server::AppState;

#[server(GetAccessToken)]
pub async fn get_access_token() -> Result<String, ServerFnError>{
    let FromContext(app_state) = extract::<FromContext<AppState>, ()>().await?;

    let tokens_guard = app_state.current_user_tokens.read().unwrap();
    match &*tokens_guard{
        Some(tokens) => Ok(tokens.access_token.clone()),
        None => {
            tracing::warn!("No access token found in appstate");
            Err(ServerFnError::ServerError("User not authenticate".to_string()))
        }
}
}

#[server(GetSpotifyUserData)]
pub async fn get_spotify_user_profile() -> Result<SpotifyUserProfile, ServerFnError>{
    tracing::info!("Attempting spotify user profile");


    let access_token = get_access_token().await?;

    let client = Client::new();
    let profile_endpoint = "https://api.spotify.com/v1/me";

    match client
        .get(profile_endpoint)
        .bearer_auth(access_token)
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success(){
                match response.json::<SpotifyUserProfile>().await {
                    Ok(profile) => {
                        tracing::info!("Succesfully fetched profile: {:?}",profile.display_name);
                        Ok(profile)
                    }
                    Err(e) => {
                        tracing::error!("failed to parse user profile json {}", e);
                        Err(ServerFnError::ServerError(format!(
                            "failed to parse spotify profile: {}",e
                        )))
                    }
                }
            } else{
                let status = response.status();
                let error_text = response.text().await.unwrap_or_else(|_| "Unknown Error".to_string());
                tracing::error!("failed to get user profile from spotify, status:{}, error:{}", status, error_text);

                Err(ServerFnError::ServerError(format!(
                    "Spotify API Error ({}): {}", status, error_text)))
            }
        }
        Err(e) => {
            tracing::error!("Network Error while fetching user profile: {}",e);
            Err(ServerFnError::ServerError(format!("Network Error: {}", e)))
        }
    }
}

#[server(GetSpotifyUserPlaylists)]
pub async fn get_spotify_user_playlists() -> Result<SpotifyPlaylistsResponse, ServerFnError>{
    tracing::info!("Attempting spotify user playlists");

    let access_token = get_access_token().await?;

    let client = Client::new();
    let mut playlist_url = reqwest::Url::parse("https://api.spotify.com/v1/me/playlists").unwrap();
    playlist_url.query_pairs_mut()
        .append_pair("limit","50");

    match client
        .get(playlist_url)
        .bearer_auth(access_token)
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success(){
                match response.json::<SpotifyPlaylistsResponse>().await {
                    Ok(playlists_data) => {
                        tracing::info!("Succesfully fetched playlists: {:?}",playlists_data.items.len());
                        Ok(playlists_data)
                    }
                    Err(e) => {
                        tracing::error!("Failed to parse playlist json: {}", e);
                        Err(ServerFnError::ServerError(format!(
                            "failed to parse spotify playlists: {}",e
                        )))
                    }
                }
            } else{
                let status = response.status();
                let error_text = response.text().await.unwrap_or_else(|_| "Unknown Error".to_string());
                tracing::error!("failed to get user playlists from spotify, status:{}, error:{}", status, error_text);

                Err(ServerFnError::ServerError(format!(
                    "Spotify API Error ({}): {}", status, error_text)))
            }
        }
        Err(e) => {
            tracing::error!("Network Error while fetching user playlists: {}",e);
            Err(ServerFnError::ServerError(format!("Network Error: {}", e)))
        }
    }


}