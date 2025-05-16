use dioxus::prelude::*;
use crate::api::{get_spotify_user_playlists_all, get_spotify_user_profile};
use crate::api_models::{SpotifyPlaylistItem, SpotifyPlaylistsResponse, SpotifyUserProfile};
use crate::components::spotify::{PlaylistsView, ProfileView};

#[component]
pub fn ShufflePage() -> Element{
    let playlists_resource : Resource<Result<Vec<SpotifyPlaylistItem>,ServerFnError>> = use_server_future(|| async{
    get_spotify_user_playlists_all().await})?;

    rsx!{
        div {class:"space-y-6",
            div { // Welcome section
                    class: "bg-gray-800 p-6 rounded-lg shadow-lg",
                    h1 { class: "text-3xl font-bold text-green-400 mb-2",
                        "Select a playlist to shuffle"
                    }
                    p { class: "text-lg text-gray-300", "True RNG shuffle..." }
                }
        // --- Playlists Section ---
            div {
                id: "playlists-section",
                class: "bg-gray-800 p-6 rounded-lg shadow-lg",
                h2 { class: "text-2xl font-semibold text-green-300 mb-3", "Your Playlists" }
                // Match block as before
                {
                    match playlists_resource.read().as_ref() {
                        Some(Ok(playlist_data)) => {
                            rsx! { PlaylistsView { playlists: playlist_data.clone() } }
                        }
                        Some(Err(e)) => {
                            rsx! { p { class: "text-red-400", "Error loading playlists: {e}" } }
                        }
                        None => {
                            rsx! { p { class: "text-yellow-400", "Loading playlists..." } }
                        }
                    }
                }
            }
        }
        
    }
}

#[component]
pub fn Home() -> Element {
    let profile_resource: Resource<Result<SpotifyUserProfile, ServerFnError>> = use_server_future( || async {
        get_spotify_user_profile().await})?;


    rsx! {
        div {class: "space-y-6", 

            div { // Welcome section
                class: "bg-gray-800 p-6 rounded-lg shadow-lg",
                h1 { class: "text-4xl font-bold text-green-400 mb-2",
                    "Better spotify because spotify is run via algos who just dont get it"
                }
                p { class: "text-lg text-gray-300", "True RNG shuffle..." }
            }

            // --- User Profile Section ---
            div {
                id: "user-profile",
                class: "bg-gray-800 p-6 rounded-lg shadow-lg",
                h2 { class: "text-2xl font-semibold text-green-300 mb-3", "Profile" }
                // Match block as before
                {
                    match profile_resource.read().as_ref() {
                        Some(Ok(profile)) => rsx! { ProfileView { profile: profile.clone() } },
                        Some(Err(e)) => rsx! { p { class: "text-red-400", "Error loading profile: {e}" } },
                        None => rsx! { p { class: "text-yellow-400", "Loading profile..." } }
                    }
                }
            }  
        }
    }
}

#[component]
pub fn LoginPage() -> Element {
    rsx! {
       div {
            class: "flex-grow flex flex-col items-center justify-center p-4", // Centers content vertically and horizontally

            div { // The "card" container for login content
                class: "bg-gray-800 p-8 md:p-12 rounded-xl shadow-2xl max-w-md w-full text-center",

                // img { src: asset!("/public/spotify_logo.svg"), alt: "Spotify Logo", class: "w-16 h-16 mx-auto mb-6" }

                h1 {
                    class: "text-3xl md:text-4xl font-bold text-green-400 mb-4",
                    "Access Your Music"
                }
                p {
                    class: "text-gray-300 mb-8 text-lg",
                    "Please log in with your Spotify account to continue to betterdSpotify and experience true playlist shuffling!"
                }

                a {
                    href: "/login", // This path is handled by your Axum server
                    class: "inline-block w-full sm:w-auto px-8 py-3 text-lg font-semibold text-white bg-green-500 rounded-lg shadow-md hover:bg-green-600 focus:outline-none focus:ring-2 focus:ring-green-400 focus:ring-opacity-75 transition-colors duration-150",
                    "Login with Spotify"
                }

                p {
                    class: "text-xs text-gray-500 mt-8",
                    "We will only ask for permission to view your playlists and create new ones. We do not store your music data beyond what's needed for the shuffle session."
                }
            }
        }
    }
    
}