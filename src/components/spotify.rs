use crate::api_models::{SpotifyPlaylistItem, SpotifyPlaylistsResponse, SpotifyUserProfile};
use dioxus::prelude::*;

#[component]
pub fn ProfileView(profile: SpotifyUserProfile) -> Element {
    rsx! {
        div {
            id: "user-profile-details",
            class: "flex items-center space-x-3", // Layout for image and text
            if let Some(images) = &profile.images {
                if let Some(image) = images.first() {
                    img {
                        src: "{image.url}",
                        class: "w-16 h-16 rounded-full object-cover border-2 border-green-500", // Styled image
                        alt: "User profile picture"
                    }
                } else {
                    div { class: "w-16 h-16 rounded-full bg-gray-700 flex items-center justify-center text-gray-400", "No Pic"}
                }
            } else {
                div { class: "w-16 h-16 rounded-full bg-gray-700 flex items-center justify-center text-gray-400", "No Pic"}
            }
            div {
                p { class: "text-xl text-gray-100", "Welcome, {profile.display_name}!" }
                p { class: "text-sm text-gray-400", "ID: {profile.id}" }
            }
        }
    }
}

#[component]
pub fn PlaylistsView(playlists: Vec<SpotifyPlaylistItem>) -> Element {
    rsx! {
        div {
            id: "playlist-list-details",
            if playlists.is_empty() {
                p { class: "text-gray-400", "You have no playlists." }
            } else {
                ul {
                    class: "space-y-3", // Adds space between playlist items
                    for playlist_item in &playlists {
                        li {
                            key: "{playlist_item.id}",
                            class: "bg-gray-700 p-3 rounded-md shadow flex items-center justify-between hover:bg-gray-600 transition-colors",
                            div { // For image and name/description
                                class: "flex items-center space-x-3",
                                if let Some(images) = &playlist_item.images {
                                    if let Some(image) = images.first() {
                                        img {
                                            src: "{image.url}",
                                            alt: "{playlist_item.name} cover",
                                            class: "w-12 h-12 object-cover rounded"
                                        }
                                    } else {
                                        div { class: "w-12 h-12 bg-gray-600 rounded flex items-center justify-center text-xs text-gray-400", "No Art"}
                                    }
                                } else {
                                    div { class: "w-12 h-12 bg-gray-600 rounded flex items-center justify-center text-xs text-gray-400", "No Art"}
                                }
                                div {
                                    p { class: "font-semibold text-gray-100", "{playlist_item.name}" }
                                    if let Some(desc) = &playlist_item.description {
                                        if !desc.is_empty() { // Only show if description exists and is not empty
                                            p { class: "text-xs text-gray-400 truncate w-64", "{desc}" } // Truncate long descriptions
                                        }
                                    }
                                }
                            }
                            // Link {
                            //     to: Route::ShufflePage { playlist_id: playlist_item.id.clone() },
                            //     class: "px-3 py-1.5 text-sm text-white bg-green-600 rounded hover:bg-green-700 focus:outline-none focus:ring-2 focus:ring-green-500 focus:ring-opacity-50",
                            //     "Shuffle"
                            // }
                        }
                    }
                }
            }
        }
    }
}
