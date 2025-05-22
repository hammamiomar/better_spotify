use dioxus::prelude::*;
use crate::api::{get_spotify_playlist_tracks_all, get_spotify_user_playlists_all, get_spotify_user_profile, shuffle_and_save_new_playlist};
use crate::api_models::{NewPlaylistDetails, SpotifyPlaylistItem, SpotifyTrackItem, SpotifyUserProfile};
use crate::components::spotify::{PlaylistsView, ProfileView};
use crate::Route;

// --- Shuffle Action Stages ---
#[derive(PartialEq, Clone, Debug)]
pub enum ShuffleStage { // Make pub if used across modules, fine for now if only in pages.rs
    Idle,
    FetchingTracks,
    ShufflingAndCreatingPlaylist { num_tracks_to_shuffle: usize },
    Completed(NewPlaylistDetails),
    Error(String),
}

// --- Stage-Specific View Components ---
#[component]
fn FetchingTracksView(playlist_name: String) -> Element {
    rsx! {
        div { class: "text-center p-4",
            div { class: "animate-spin rounded-full h-12 w-12 border-t-4 border-b-4 border-green-500 mx-auto mb-4" }
            p { class: "text-xl text-yellow-400", "Fetching all tracks for \"{playlist_name}\"..." }
            p { class: "text-sm text-gray-400 mt-2", "This might take a moment for very large playlists." }
        }
    }
}

#[component]
fn ShufflingAndCreatingView(playlist_name: String, num_tracks: usize) -> Element {
    rsx! {
        div { class: "text-center p-4",
            div { class: "animate-spin rounded-full h-12 w-12 border-t-4 border-b-4 border-purple-500 mx-auto mb-4" }
            p { class: "text-xl text-yellow-400",
                "Processing {num_tracks} tracks for \"{playlist_name}\"."
            }
            p { class: "text-sm text-gray-400 mt-2", "Shuffling and creating your new playlist..."}
        }
    }
}

#[component]
fn ShuffleCompleteView(details: NewPlaylistDetails) -> Element {
    rsx! {
        div { class: "text-center p-4",
            p { class: "text-2xl text-green-500 mb-3", "＼(＾▽＾)／ True Shuffle Complete! ＼(＾▽＾)／" }
            p { class: "text-gray-200 mb-1", "New playlist created:" }
            p { class: "text-xl font-semibold text-gray-100 mb-4", "\"{details.name}\"" }
            a {
                href: "{details.external_url}", target: "_blank", rel: "noopener noreferrer",
                class: "inline-block px-6 py-3 text-white bg-spotify-green rounded-lg hover:bg-opacity-80 shadow-md", // Define bg-spotify-green or use existing
                "Open New Playlist on Spotify"
            }
            // Optional: Button to go back to shuffle selection or home
            // Link { to: Route::ShuffleSelectPage {}, class: "mt-6 inline-block text-sm text-blue-400 hover:underline", "Shuffle Another?" }
        }
    }
}

#[component]
fn ShuffleErrorView(error_message: String, on_retry: EventHandler<()>) -> Element {
    rsx! {
        div { class: "text-center p-4",
            p { class: "text-2xl text-red-500 mb-3", "(╯°□°）╯︵ ┻━┻ Shuffle Process Failed!" }
            p { class: "text-gray-300 mt-2 mb-6", "{error_message}" }
            button {
                class: "px-6 py-2 text-white bg-orange-500 rounded-lg hover:bg-orange-600 shadow",
                onclick: move |_| on_retry.call(()),
                "Try Again"
            }
        }
    }
}

// --- ShuffleActionPage (Orchestrator) ---
#[component]
pub fn ShuffleActionPage(playlist_id: String, playlist_name: String) -> Element {
    let mut current_stage = use_signal(|| ShuffleStage::Idle);
    // This signal will store the fetched tracks to pass to the shuffle_and_save function
    // This ensures tracks are fetched only ONCE.
    let mut fetched_tracks_for_shuffle: Signal<Option<Vec<SpotifyTrackItem>>> = use_signal(|| None);

    // Clones for async tasks
    let pid_for_tasks = playlist_id.clone();
    let pname_for_tasks = playlist_name.clone();

    // Effect to manage the shuffle workflow
    use_effect(move || {
        match *current_stage.read() {
            ShuffleStage::FetchingTracks => {
                let mut stage_signal = current_stage;
                let mut tracks_signal = fetched_tracks_for_shuffle;
                let pid_clone = pid_for_tasks.clone();

                spawn(async move { // dioxus::prelude::spawn
                    match get_spotify_playlist_tracks_all(pid_clone).await {
                        Ok(tracks) => {
                            if tracks.is_empty() {
                                stage_signal.set(ShuffleStage::Error("Selected playlist is empty or no tracks were found.".to_string()));
                            } else {
                                let num_tracks = tracks.len();
                                tracks_signal.set(Some(tracks)); // Store fetched tracks
                                // Transition to next stage, now with num_tracks from the fetched data
                                stage_signal.set(ShuffleStage::ShufflingAndCreatingPlaylist { num_tracks_to_shuffle: num_tracks });
                            }
                        }
                        Err(e) => stage_signal.set(ShuffleStage::Error(format!("Failed to fetch tracks: {}", e))),
                    }
                });
            }
            ShuffleStage::ShufflingAndCreatingPlaylist { num_tracks_to_shuffle: _ } => { // num_tracks already set for UI
                // Check if tracks are actually fetched and stored
                if let Some(tracks) = fetched_tracks_for_shuffle.read().as_ref() {
                    // If `shuffle_and_save_new_playlist` needs the track URIs directly,
                    // extract them here. Otherwise, it might re-fetch based on ID if that's its design.
                    // For now, assuming `shuffle_and_save_new_playlist` takes playlist_id and re-fetches or uses cached if available.
                    // If `shuffle_and_save_new_playlist` is modified to take `Vec<String>` of track URIs,
                    // you would extract them from `tracks` here.

                    let mut stage_signal = current_stage;
                    let mut pid_clone = pid_for_tasks.clone();
                    let mut pname_clone = pname_for_tasks.clone();
                    // IMPORTANT: The current `shuffle_and_save_new_playlist` re-fetches tracks.
                    // If you want to avoid re-fetching, modify `shuffle_and_save_new_playlist`
                    // to accept `Vec<SpotifyTrackItem>` or `Vec<String>` (track URIs) as an argument.
                    // For this example, we proceed with its current signature.

                    spawn(async move {
                        match shuffle_and_save_new_playlist(pid_clone, pname_clone).await {
                            Ok(details) => stage_signal.set(ShuffleStage::Completed(details)),
                            Err(e) => stage_signal.set(ShuffleStage::Error(format!("Failed to shuffle and save playlist: {}", e))),
                        }
                    });
                } else {
                    // This case should ideally not be reached if FetchingTracks succeeded.
                    let mut stage_signal = current_stage;
                    stage_signal.set(ShuffleStage::Error("Track data was lost before shuffling could start. Please retry.".to_string()));
                }
            }
            _ => {} // Do nothing for Idle, Completed, Error in this effect
        }
    });

    rsx! {
        div {
            class: "p-4 md:p-8 text-center",
            div { /* Title div */
                class: "mb-8",
                h1 { class: "text-3xl md:text-4xl font-bold text-green-400 mb-1", "Shuffling:" }
                h2 { class: "text-2xl md:text-3xl font-semibold text-gray-200", "\"{playlist_name}\"" }
                p { class: "text-sm text-gray-400", "(ID: {playlist_id})" }
            }

            div { // Main content area for stages
                class: "bg-gray-800 p-6 rounded-lg shadow-lg max-w-xl mx-auto min-h-[12rem] flex flex-col items-center justify-center",
                match &*current_stage.read() {
                    ShuffleStage::Idle => rsx! {
                        button {
                            class: "px-8 py-4 text-xl font-semibold text-white bg-purple-600 rounded-lg shadow hover:bg-purple-700 focus:outline-none focus:ring-2 focus:ring-purple-400 focus:ring-opacity-75",
                            onclick: move |_| {
                                fetched_tracks_for_shuffle.set(None); // Clear previous tracks
                                current_stage.set(ShuffleStage::FetchingTracks);
                            },
                            "Start True Shuffle Process!"
                        }
                    },
                    ShuffleStage::FetchingTracks => rsx! {
                        FetchingTracksView { playlist_name: playlist_name.clone() }
                    },
                    ShuffleStage::ShufflingAndCreatingPlaylist { num_tracks_to_shuffle } => rsx! {
                        ShufflingAndCreatingView { playlist_name: playlist_name.clone(), num_tracks: *num_tracks_to_shuffle }
                    },
                    ShuffleStage::Completed(ref details) => rsx! {
                        ShuffleCompleteView { details: details.clone() }
                    },
                    ShuffleStage::Error(ref err_msg) => rsx! {
                        ShuffleErrorView {
                            error_message: err_msg.clone(),
                            on_retry: move |_| {
                                fetched_tracks_for_shuffle.set(None); // Clear previous tracks before retry
                                current_stage.set(ShuffleStage::Idle);
                            }
                        }
                    }
                }
            }
        }
    }
}
