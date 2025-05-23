use std::{vec, collections::HashMap};
use dioxus::prelude::*;

use reqwest::Client;
use crate::api_models::{NewPlaylistDetails, SpotifyPlaylistItem, SpotifyPlaylistTrackResponse, SpotifyPlaylistsResponse, SpotifyTrackItem, SpotifyUserProfile};

#[cfg(feature="server")]
use crate::server::AppState;

#[cfg(feature="server")]
use rand::{thread_rng, seq::SliceRandom};


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

#[server(GetSpotifyUserId)]
pub async fn get_spotify_user_id() -> Result<String, ServerFnError>{
    let profile = get_spotify_user_profile().await?;
    Ok(profile.id)
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
pub async fn get_spotify_playlist_tracks_all(playlist_id: String) -> Result<Vec<SpotifyTrackItem>,ServerFnError>{
    tracing::info!("Attempting to get tracks for playlist:{}",playlist_id);
    

    let mut all_tracks: Vec<SpotifyTrackItem> = vec![];
    let mut page_tracks: Vec<SpotifyTrackItem> = vec![];
    let mut current_offset: u32 = 0;
    let LIMIT :u32 = 50;
    
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

#[server(GetSpotifyPlaylist)]
pub async fn get_spotify_playlist(playlist_id: String) -> Result<SpotifyPlaylistItem, ServerFnError> {
    tracing::info!("Attempting to get playlist details for ID: {}", playlist_id);
    
    let access_token = get_access_token().await?;
    
    let client = Client::new();
    let playlist_url = format!("https://api.spotify.com/v1/playlists/{}", playlist_id);
    
    match client
        .get(&playlist_url)
        .bearer_auth(access_token)
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<SpotifyPlaylistItem>().await {
                    Ok(playlist) => {
                        tracing::info!("Successfully fetched playlist: {}", playlist.name);
                        Ok(playlist)
                    }
                    Err(e) => {
                        tracing::error!("Failed to parse playlist json: {}", e);
                        Err(ServerFnError::ServerError(format!(
                            "Failed to parse Spotify playlist: {}", e
                        )))
                    }
                }
            } else {
                let status = response.status();
                let error_text = response.text().await.unwrap_or_else(|_| "Unknown Error".to_string());
                tracing::error!("Failed to get playlist from Spotify, status:{}, error:{}", status, error_text);
                
                Err(ServerFnError::ServerError(format!(
                    "Spotify API Error ({}): {}", status, error_text)))
            }
        }
        Err(e) => {
            tracing::error!("Network Error while fetching playlist: {}", e);
            Err(ServerFnError::ServerError(format!("Network Error: {}", e)))
        }
    }
}

#[server(GetSpotifyPlaylistTracksPage)]
pub async fn get_spotify_playlist_tracks_page(playlist_id: String, limit: u32, offset:u32) -> Result<SpotifyPlaylistTrackResponse, ServerFnError>{
    tracing::info!("Attempting spotify playlist tracks page offset: {}", offset);

    let access_token = get_access_token().await?;

    const FIELDS: &str = "items(track(id,name,uri)),limit,offset,total,next";

    let client = Client::new();
    let mut tracks_url = reqwest::Url::parse(
        format!("https://api.spotify.com/v1/playlists/{}/tracks",playlist_id).as_str()).unwrap();
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

#[server(ShuffleAndSavePlaylist)] // Reverting to this name
pub async fn shuffle_and_save_new_playlist(
    original_playlist_id: String,
    original_playlist_name: String,
) -> Result<NewPlaylistDetails, ServerFnError> { // Use default ServerFnError for now
    #[cfg(feature = "server")]
    {
        tracing::info!("API: Server-side shuffle for playlist: '{}' (ID: {})", original_playlist_name, original_playlist_id);

        // 1. Get Current User's Spotify ID
        let user_id = match get_spotify_user_id().await {
            Ok(id) => id,
            Err(e) => return Err(ServerFnError::ServerError(format!("Failed to get user ID: {}", e))),
        };
        tracing::info!("API: Target user ID: {}", user_id);

        // 2. Fetch All Tracks for the Original Playlist
        let tracks_for_shuffling = match get_spotify_playlist_tracks_all(original_playlist_id.clone()).await {
            Ok(tracks) => tracks,
            Err(e) => return Err(ServerFnError::ServerError(format!("Failed to fetch tracks for '{}': {}", original_playlist_name, e))),
        };

        if tracks_for_shuffling.is_empty() {
            return Err(ServerFnError::ServerError(format!("Playlist '{}' is empty.", original_playlist_name)));
        }
        tracing::info!("API: Fetched {} tracks for '{}'.", tracks_for_shuffling.len(), original_playlist_name);

        // 3. Extract Track URIs AND THEN Shuffle them
        let mut track_uris: Vec<String> = tracks_for_shuffling
            .into_iter()
            .filter_map(|t| t.id.map(|id_val| format!("spotify:track:{}", id_val)))
            .collect();

        if track_uris.is_empty() {
            return Err(ServerFnError::ServerError("No valid track URIs found in the playlist.".to_string()));
        }

        // --- Perform shuffle synchronously here ---
        { // Create a limited scope for rng if using thread_rng
            let mut rng = thread_rng(); // Create RNG here
            track_uris.shuffle(&mut rng); // Shuffle the mutable vector
        } // rng goes out of scope; if it was thread_rng, its non-Send parts are dropped.
          // If thread_rng() still gives Send issues due to the outer async fn,
          // use `let mut rng = rand::rngs::StdRng::from_entropy();`
        tracing::info!("API: Shuffled {} track URIs.", track_uris.len());
        // --- End shuffle ---


        // 4. Create a New Playlist
        let access_token = match get_access_token().await { // Get token again for subsequent calls
            Ok(token) => token,
            Err(e) => return Err(e),
        };
        let client = Client::new();
        let new_playlist_name = format!("{} - TRUE SHUFFLED", original_playlist_name);

        #[derive(serde::Serialize)]
        struct CreatePlaylistPayload<'a> { name: &'a str, public: bool, description: String }
        let create_payload = CreatePlaylistPayload {
            name: &new_playlist_name,
            public: false,
            description: format!(
                "A true random shuffle of '{}'!",
                original_playlist_name
            ),
        };
        let create_playlist_url = format!("https://api.spotify.com/v1/users/{}/playlists", user_id);
        tracing::info!("API: Creating new playlist: {}", new_playlist_name);

        let created_playlist_data: SpotifyPlaylistItem = match client
            .post(&create_playlist_url)
            .bearer_auth(access_token.clone())
            .json(&create_payload)
            .send().await {
                // Explicit match for Result<reqwest::Response, reqwest::Error>
                Ok(response) => {
                    if response.status().is_success() || response.status().as_u16() == 201 {
                        match response.json::<SpotifyPlaylistItem>().await {
                            Ok(data) => data,
                            Err(e) => return Err(ServerFnError::ServerError(format!("API: Parse new playlist response error: {}", e))),
                        }
                    } else {
                        let s = response.status(); let t = response.text().await.unwrap_or_default();
                        return Err(ServerFnError::ServerError(format!("API: Spotify error {} creating playlist: {}", s, t)));
                    }
                }
                Err(e) => return Err(ServerFnError::ServerError(format!("API: Network error creating playlist: {}", e))),
            };
        let new_playlist_id = created_playlist_data.id.clone();
        tracing::info!("API: New playlist created '{}' (ID: {})", new_playlist_name, new_playlist_id);

        // 5. Add Shuffled Tracks to the New Playlist (in batches)
        if !track_uris.is_empty() {
            let add_tracks_url_base = format!("https://api.spotify.com/v1/playlists/{}/tracks", new_playlist_id);
            for chunk_of_uris in track_uris.chunks(100) {
                 #[derive(serde::Serialize)]
                struct AddTracksPayload<'a> {
                    uris: &'a [String],
                }
                let add_payload = AddTracksPayload {
                    uris: chunk_of_uris,
                };
                tracing::info!(
                    "API: Adding {} tracks to new playlist ID {}",
                    chunk_of_uris.len(),
                    new_playlist_id
                );
                match client.post(&add_tracks_url_base).bearer_auth(access_token.clone()).json(&add_payload).send().await {
                    Ok(_response) => { /* Check status */ }
                    Err(e) => return Err(ServerFnError::ServerError(format!("API: Network error adding tracks: {}",e))),
                }
                if track_uris.len() > 100 && chunk_of_uris.len() == 100 { // Avoid sleep if only one chunk or last small chunk
                    tokio::time::sleep(tokio::time::Duration::from_millis(250)).await;
                }
            }
        }
        tracing::info!("API: All tracks added to new playlist: {}", new_playlist_name);

        // 6. Copy the original playlist's image to the new playlist
        // First, get the original playlist details to access its images
        match get_spotify_playlist(original_playlist_id.clone()).await {
            Ok(original_playlist) => {
                if let Some(images) = &original_playlist.images {
                    if !images.is_empty() {
                        // Typically we want the first image (usually the largest)
                        let original_image_url = &images[0].url;
                        tracing::info!("API: Copying image from original playlist: {}", original_image_url);
                        
                        // Fetch the image from the URL
                        match client.get(original_image_url).send().await {
                            Ok(img_response) => {
                                if img_response.status().is_success() {
                                    // Get the image bytes
                                    match img_response.bytes().await {
                                        Ok(img_bytes) => {
                                            // Convert to base64 (required by Spotify API)
                                            let base64_img = base64::encode(img_bytes);
                                            
                                            // Call Spotify API to update playlist image
                                            let upload_image_url = format!("https://api.spotify.com/v1/playlists/{}/images", new_playlist_id);
                                            
                                            match client
                                                .put(&upload_image_url)
                                                .bearer_auth(access_token.clone())
                                                .header("Content-Type", "image/jpeg")
                                                .body(base64_img)
                                                .send()
                                                .await
                                            {
                                                Ok(upload_response) => {
                                                    if upload_response.status().is_success() {
                                                        tracing::info!("API: Successfully copied image to new playlist");
                                                    } else {
                                                        let status = upload_response.status();
                                                        let err_text = upload_response.text().await.unwrap_or_default();
                                                        tracing::error!("API: Failed to upload image: {} - {}", status, err_text);
                                                    }
                                                },
                                                Err(e) => {
                                                    tracing::error!("API: Network error uploading image: {}", e);
                                                }
                                            }
                                        },
                                        Err(e) => {
                                            tracing::error!("API: Failed to get image bytes: {}", e);
                                        }
                                    }
                                } else {
                                    tracing::error!("API: Failed to fetch image, status: {}", img_response.status());
                                }
                            },
                            Err(e) => {
                                tracing::error!("API: Network error fetching image: {}", e);
                            }
                        }
                    } else {
                        tracing::info!("API: Original playlist has no images");
                    }
                } else {
                    tracing::info!("API: Original playlist has no images array");
                }
            },
            Err(e) => {
                tracing::error!("API: Failed to get original playlist details: {}", e);
            }
        }
        
        // 7. Return Success
        let web_url = format!("https://open.spotify.com/playlist/{}", new_playlist_id);
        Ok(NewPlaylistDetails {
            id: new_playlist_id,
            name: new_playlist_name,
            external_url: web_url,
        })
    }
}