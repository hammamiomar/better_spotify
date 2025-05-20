use crate::api_models::{SpotifyPlaylistItem, SpotifyUserProfile};
use dioxus::prelude::*;
use log::info;

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
pub fn PlaylistsView(playlists: Vec<SpotifyPlaylistItem>, selected_playlist: Signal<Option<SpotifyPlaylistItem>>) -> Element {
    let currently_selected_id_opt: Option<String> = selected_playlist
        .read()
        .as_ref()
        .map(|p| p.id.clone());

    rsx! {
        div {
            id: "playlist-list-details",
                ul {
                    class: "space-y-2 max-h-96 overflow-y-auto",
                    if playlists.is_empty(){
                        li{class: "text-gray-400 p-3 text-center", "No playlists to display"}
                    }else{
                        {playlists.iter().map(|playlist_item| {
                            let is_selected = match &currently_selected_id_opt {
                                Some(selected_id_str) => &playlist_item.id == selected_id_str,
                                None => false,
                            };

                            let item_classes = if is_selected {
                                "bg-green-700 p-3 rounded-md shadow flex items-center justify-between transition-colors cursor-pointer"
                            } else {
                                "bg-gray-700 p-3 rounded-md shadow flex items-center justify-between hover:bg-gray-600 transition-colors cursor-pointer"
                            };

                            let item_for_click_closure = playlist_item.clone();
                            let mut signal_for_click_closure = selected_playlist;

                            rsx! {
                                li {
                                    key: "{playlist_item.id}",
                                    class: "{item_classes}",
                                    onclick: move |_| {
                                        //log::info!("clicked playlistL{}",item_for_click_closure.id);
                                        signal_for_click_closure.set(Some(item_for_click_closure.clone()));
                                    },
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
                                                if !desc.is_empty() { 
                                                    p { class: "text-xs text-gray-400 truncate w-64", "{desc}" } 
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        })}
                    } 
                }
            }
        }
    }

