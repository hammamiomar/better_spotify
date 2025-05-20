use std::{vec, collections::HashMap};

use dioxus::prelude::*;
use reqwest::Client;
use crate::api_models::{SpotifyPlaylistItem, SpotifyPlaylistTrackResponse, SpotifyPlaylistsResponse, SpotifyTokenResponse, SpotifyTrackItem, SpotifyUserProfile};

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

#[server(GetSpotifyUserPlaylistsPage)]
pub async fn get_spotify_user_playlists_page(limit: u32, offset:u32) -> Result<SpotifyPlaylistsResponse, ServerFnError>{
    tracing::info!("Attempting spotify user playlists page offset: {}", offset);

    let access_token = get_access_token().await?;

    let client = Client::new();
    let mut playlist_url = reqwest::Url::parse("https://api.spotify.com/v1/me/playlists").unwrap();
    playlist_url.query_pairs_mut()
        .append_pair("limit",&limit.to_string())
        .append_pair("offset", &offset.to_string());

    match client
        .get(playlist_url)
        .bearer_auth(access_token)
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success(){
                match response.json::<SpotifyPlaylistsResponse>().await {
                    Ok(page_data) => {
                        tracing::info!(
                            "Successfully fetched page of {} playlists. Offset: {}",
                            page_data.items.len(),
                            page_data.offset);
                        Ok(page_data)
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

#[server(GetSpotifyUserPlaylistsAll)]
pub async fn get_spotify_user_playlists_all() -> Result<Vec<SpotifyPlaylistItem>, ServerFnError>{
    
    let mut all_playlists:Vec<SpotifyPlaylistItem> = vec![];

    let mut current_limit: u32 = 50;
    let mut current_offset: u32 = 0;

    loop {
        match get_spotify_user_playlists_page(current_limit, current_offset).await {
            Ok(page_response) =>{
                // if page_response.items.is_empty() && page_response.next.is_none(){
                //     tracing::info!("No more playlists to fetch.");
                //     break;
                // }
                let num_items = page_response.items.len();
                all_playlists.extend(page_response.items);
                tracing::info!("Page: {}, Fetched {} playlists, total now: {}. Offset was: {}, total playlists to fetch:{}",
                    current_offset/50,
                    num_items,
                    all_playlists.len(),
                    current_offset,
                    page_response.total
                );

               let next_url = match page_response.next {
                    Some(s) => s,
                    None => break
                };
                let url = reqwest::Url::parse(&next_url)?;
                let params: HashMap<_,_> = url.query_pairs().into_owned().collect();


                current_offset = match params.get("offset") {
                    Some(offset_str) => match offset_str.parse::<u32>() {
                        Ok(num) => num,
                        Err(e) => {
                            tracing::error!("Failed to parse 'offset' from next_url query ('{}'): {}", offset_str, e);
                            return Err(ServerFnError::ServerError(format!("Invalid 'offset' in next URL: {}", e)));
                        }
                    },
                    None => {
                        tracing::warn!("'offset' not found in next_url query: {}. Assuming end or error.", next_url);
                        return Err(ServerFnError::ServerError("Missing 'offset' in Spotify's next URL".to_string()));
                    }
                };
                current_limit = match params.get("limit") {
                    Some(limit_str) => match limit_str.parse::<u32>() {
                        Ok(num) => num,
                        Err(e) => {
                            tracing::error!("Failed to parse 'offset' from next_url query ('{}'): {}", limit_str, e);
                            return Err(ServerFnError::ServerError(format!("Invalid 'offset' in next URL: {}", e)));
                        }
                    },
                    None => {
                        tracing::warn!("'offset' not found in next_url query: {}. Assuming end or error.", next_url);
                        return Err(ServerFnError::ServerError("Missing 'offset' in Spotify's next URL".to_string()));
                    }
                };
            
            }
            Err(e) =>{
                tracing::error!("error fetcing page of playlists: {}",e);
                return Err(e)
            }
        }
    }

    let mut unique_checker = std::collections::HashSet::new(); 
    all_playlists.retain(|p| unique_checker.insert(p.id.clone()));
    tracing::info!("Finished fetching. Total playlists retrieved: {}", all_playlists.len());
    Ok(all_playlists)
}

#[server(GetSpotifyPlaylistTracksAll)]
pub async fn get_spotify_playlist_tracks_all(playlist_item: SpotifyPlaylistItem) -> Result<Vec<SpotifyTrackItem>,ServerFnError>{
    tracing::info!("Attempting to get tracks for playlist:{}",playlist_item.name);
    

    let mut all_tracks: Vec<SpotifyTrackItem> = vec![];
    let mut page_tracks: Vec<SpotifyTrackItem> = vec![];
    let mut current_offset: u32 = 0;
    let LIMIT :u32 = 50;
    let playlist_id = playlist_item.id;
    
    loop {
        match get_spotify_playlist_tracks_page(playlist_id.clone(), LIMIT, current_offset).await {
            Ok(page_response) => {
                if page_response.items.is_empty() && page_response.next.is_none(){
                    tracing::info!("No more tracks on this page and no more next page");
                    break;
                }
                let num_items = page_response.items.len();
                page_tracks = page_response.items
                    .into_iter()
                    .filter_map(|item_wrapper| item_wrapper.track)
                    .collect();
                all_tracks.extend(page_tracks);

                tracing::info!("Page: {}, Fetched {} tracks, total now: {}. Offset was: {}, total playlists to fetch:{}",
                                    current_offset/LIMIT,
                                    num_items,
                                    all_tracks.len(),
                                    current_offset,
                                    page_response.total
                                );

                let next_url = match page_response.next {
                    Some(s) => s,
                    None => break
                };
                let url = reqwest::Url::parse(&next_url)?;
                let params: HashMap<_,_> = url.query_pairs().into_owned().collect();


                current_offset = match params.get("offset") {
                    Some(offset_str) => match offset_str.parse::<u32>() {
                        Ok(num) => num,
                        Err(e) => {
                            tracing::error!("Failed to parse 'offset' from next_url query ('{}'): {}", offset_str, e);
                            return Err(ServerFnError::ServerError(format!("Invalid 'offset' in next URL: {}", e)));
                        }
                    },
                    None => {
                        tracing::warn!("'offset' not found in next_url query: {}. Assuming end or error.", next_url);
                        return Err(ServerFnError::ServerError("Missing 'offset' in Spotify's next URL".to_string()));
                    }
                };

            }
            Err(e) => {
                tracing::error!("Error fetching page of tracks:{}",e);
                return Err(e)
            }
        }
    }
    tracing::info!("Finished fetching. Total tracks retrieved: {}", all_tracks.len());
    Ok(all_tracks)

}
#[server(GetSpotifyPlaylistTracksPage)]
pub async fn get_spotify_playlist_tracks_page(playlist_id: String, limit: u32, offset:u32) -> Result<SpotifyPlaylistTrackResponse, ServerFnError>{
    tracing::info!("Attempting spotify playlist tracks page offset: {}", offset);

    let access_token = get_access_token().await?;

    const FIELDS: &str = "items(track(id,name,uri,artists(name),album(name,images),duration_ms,explicit)),limit,offset,total,next";

    let client = Client::new();
    let mut tracks_url = reqwest::Url::parse(
        format!("https://api.spotify.com/playlists/{}/tracks",playlist_id).as_str()).unwrap();
    tracks_url.query_pairs_mut()
        .append_pair("offset", &offset.to_string())
        .append_pair("limit", &limit.to_string())
        .append_pair("fields", FIELDS);


    match client
        .get(tracks_url)
        .bearer_auth(access_token)
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success(){
                match response.json::<SpotifyPlaylistTrackResponse>().await {
                    Ok(page_data) => {
                        tracing::info!(
                            "Successfully fetched page of {} tracks. Offset: {}",
                            page_data.items.len(),
                            page_data.offset);
                        Ok(page_data)
                    }
                    Err(e) => {
                        tracing::error!("Failed to parse playlist json: {}", e);
                        Err(ServerFnError::ServerError(format!(
                            "failed to parse spotify playlists tracks: {}",e
                        )))
                    }
                }
            } else{
                let status = response.status();
                let error_text = response.text().await.unwrap_or_else(|_| "Unknown Error".to_string());
                tracing::error!("failed to get playlist tracks from spotify, status:{}, error:{}", status, error_text);

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
