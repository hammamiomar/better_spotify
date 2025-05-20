use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct SpotifyTokenResponse {
    pub access_token: String,
    token_type: String,
    scope: String,
    expires_in: u64,
    refresh_token: Option<String>,
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SpotifyUserProfile {
    pub display_name: String,
    pub id: String,
    pub images: Option<Vec<SpotifyImageObject>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SpotifyImageObject {
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
    pub uri: String, // Add owner, public, collaborative, tracks url etc. if needed
}

// For the /me/playlists endpoint top-level response
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SpotifyPlaylistsResponse {
    pub items: Vec<SpotifyPlaylistItem>,
    pub href: String,
    pub limit: u32,
    pub next: Option<String>,
    pub offset: u32,
    pub previous: Option<String>,
    pub total: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SpotifyPlaylistTrackResponse {
    pub items: Vec<PlaylistItemTrackWrapper>,
    pub href: String,
    pub limit: u32,
    pub next: Option<String>,
    pub offset: u32,
    pub previous: Option<String>,
    pub total: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PlaylistItemTrackWrapper {
    pub track: Option<SpotifyTrackItem>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SpotifyTrackItem {
    pub id: Option<String>,
    pub uri: String,
    pub name: String,
    pub artists: Vec<SpotifyTrackArtistsSimple>,
    pub album: SpotifyTrackAlbumSimple,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SpotifyTrackAlbumSimple {
    pub id: Option<String>,
    pub name: String,
    pub images: Option<SpotifyImageObject>,
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SpotifyTrackArtistsSimple {
    pub id: Option<String>,
    pub name: String,
}
