use dioxus::prelude::*;
use crate::api::{get_spotify_user_playlists_all, get_spotify_user_profile};
use crate::api_models::{SpotifyPlaylistItem, SpotifyUserProfile};
use crate::components::spotify::{PlaylistsView, ProfileView};
use crate::Route;

#[component]
pub fn ShufflePage() -> Element{
    let playlists_resource : Resource<Result<Vec<SpotifyPlaylistItem>,ServerFnError>> = use_server_future(|| async{
    get_spotify_user_playlists_all().await})?;
    let mut search_term = use_signal(String::new);
    let selected_playlist : Signal<Option<SpotifyPlaylistItem>> = use_signal(|| None);

    let navigator = use_navigator();

    rsx!{
        div {class:"space-y-6 p-4 md:p-8",
            div { // Welcome section
                    class: "bg-gray-800 p-6 rounded-lg shadow-lg",
                    h1 { class: "text-3xl font-bold text-green-400 mb-2",
                        "Playlist Shuffler Studio"
                    }
                    p { class: "text-lg text-gray-300", "Select or search for a playlist to begin..." }
                }
            //Search
            div {
                class: "my-4 p-4 bg-gray-700 rounded-lg shadow",
                input {
                    r#type: "text",
                    placeholder: "Search your playlists...",
                    class: "w-full p-3 bg-gray-800 text-gray-100 rounded-md focus:ring-2 focus:ring-green-500 focus:outline-none",
                    value: "{search_term}", // Bind input value to the signal
                    oninput: move |evt| search_term.set(evt.value()), // Update signal on input
                }
            }
        // --- Playlists Section ---
            div {
                id: "shuffle-playlist-selection-list",
                class: "bg-gray-800 p-4 md:p-6 rounded-lg shadow-lg",
                h2 { class: "text-2xl font-semibold text-green-300 mb-4 border-b border-gray-700 pb-2", "Your Playlists" }
                {
                    match playlists_resource.read().as_ref() {
                        Some(Ok(all_playlists_vec)) => {
                            // Filter playlists based on search_term
                            let filtered_playlists = {
                                let search_lower = search_term.read().to_lowercase();
                                if search_lower.is_empty() {
                                    all_playlists_vec.clone() // No filter, show all (clone for iteration)
                                } else {
                                    all_playlists_vec.iter().filter(move |p| {
                                        p.name.to_lowercase().contains(&search_lower)
                                    }).cloned().collect::<Vec<SpotifyPlaylistItem>>()
                                }
                            };

                            if filtered_playlists.is_empty() && !search_term.read().is_empty() {
                                rsx! { p { class: "text-gray-400 text-center py-4", "No playlists match your search."}}
                            } else if filtered_playlists.is_empty() {
                                rsx! { p { class: "text-gray-400 text-center py-4", "No playlists found."}}
                            } else {
                                // Pass down the selected_playlist signal and filtered list
                                rsx!{PlaylistsView {
                                    playlists: filtered_playlists,
                                    selected_playlist: selected_playlist // Pass the signal
                                }}
                            }
                        }
                        Some(Err(e)) => {
                            rsx! { p { class: "text-red-400 text-center py-4", "Error loading playlists: {e}" } }
                        }
                        None => {rsx! {p { "Loading playlists..."}}}
                    }
                }
            }
            div {
                class: "mt-6 text-center",
                button {
                    disabled: selected_playlist.read().is_none(), // Enable only if a playlist is selected
                    class: "px-6 py-3 text-lg font-semibold text-white bg-blue-600 rounded-lg shadow hover:bg-blue-700 disabled:opacity-50 disabled:bg-gray-500 disabled:cursor-not-allowed transition-opacity",
                    onclick: move |_| {
                        if let Some(playlist) = selected_playlist.read().as_ref() {
                            navigator.push(Route::ShuffleActionPage {
                                playlist_id: playlist.id.clone(),
                                playlist_name: playlist.name.clone(),
                            });
                        }
                    },
                    if selected_playlist.read().is_some() {
                        "Prepare Shuffle for \"{selected_playlist.read().as_ref().unwrap().name}\""
                    } else {
                        "Select a Playlist to Prepare Shuffle"
                    }
                }
            }

        }
        
    }
}

#[component]
pub fn Home() -> Element {
    let is_authenticated = use_server_future( || async {
        crate::api::check_auth().await})?;

    rsx! {
        div {class: "space-y-6", 

            div { // Welcome section
                class: "bg-gray-800 p-6 rounded-lg shadow-lg",
                h1 { class: "text-8xl font-bold text-green-400 mb-2",
                    "Better spotify because spotify is run via algos who just dont get it"
                }
                wbr {  }
                p { class: "text-3xl text-purple-400 font-bold mb-2", "I AM OFFERING YOU A TRUE RNG SHUFFLE" }
                wbr {  }
                p { class: "text-2xl text-gray-300 underline ", "NO SPOTIFY I DO NOT WANT TO SHUFFLE MY 6000 SONG PLAYLIST
                 AND GET THE SAME SONGS EVERYTIME BECAUSE YOU THINK THOSE ARE MY FAVORITE SONGS -- ONLY BECAUSE YOU ALWAYS GIVE THEM TO ME" }            }

            // --- User Profile Section ---
            match is_authenticated.read().as_ref() {
                Some(Ok(true)) => rsx! {
                    // Show authenticated content
                    AuthenticatedUserProfile {}
                },
                Some(Ok(false)) => rsx! {
                    div {
                        class: "bg-gray-800 p-6 rounded-lg shadow-lg text-center",
                        h2 { class: "text-2xl font-semibold text-green-300 mb-3", "Get Started" }
                        p { class: "text-gray-300 mb-4", "Login with Spotify to start shuffling your playlists!" }
                        Link {
                            to: Route::LoginPage {},
                            class: "inline-block px-8 py-3 text-lg font-semibold text-white bg-green-500 rounded-lg shadow-md hover:bg-green-600 transition-colors",
                            "Login with Spotify"
                        }
                    }
                },
                _ => rsx! {}  // Loading or error state
            }
        }
    }
}

#[component]
fn AuthenticatedUserProfile() -> Element {
    let profile_resource: Resource<Result<SpotifyUserProfile, ServerFnError>> = use_server_future( || async {
        get_spotify_user_profile().await})?;
    
    rsx! {
        div {
            id: "user-profile",
            class: "bg-gray-800 p-6 rounded-lg shadow-lg",
            h2 { class: "text-2xl font-semibold text-green-300 mb-3", "Profile" }
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
                    href: "/auth/spotify", // This path is handled by your Axum server
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