use dioxus::prelude::*;
use serde::{Deserialize,Serialize};



#[derive(Deserialize, Debug)]
pub struct SpotifyTokenResponse{
    pub access_token:String,
    token_type:String,
    scope:String,
    expires_in: u64,
    refresh_token : Option<String>,
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SpotifyUserProfile{
    pub display_name: String,
    pub id: String,
    pub images: Option<Vec<SpotifyImageObject>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SpotifyImageObject{
    pub url: String,
    pub height: Option<u32>,
    pub width: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SpotifyPlaylistItem {
    pub id: String,
    pub name: String,
    pub images: Option<Vec<SpotifyImageObject>>,
    pub description: Option<String>,
    // Add owner, public, collaborative, tracks url etc. if needed
}

// For the /me/playlists endpoint top-level response
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SpotifyPlaylistsResponse {
    pub items: Vec<SpotifyPlaylistItem>,
    // Add total, limit, offset, etc. if needed
}