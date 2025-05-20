use dioxus::prelude::*;
use crate::{api::get_access_token, Route};

#[component]
pub fn NavBar() -> Element {
    let has_token = use_server_future( || async {
        get_access_token().await})?;

    rsx! {
        header {
            class: "bg-gray-800 text-white p-4 shadow-md",
            nav {
                class: "container mx-auto flex justify-between items-center",

                div{class: "text-xl font-bold hover:text-green-400",
                     Link {to:Route::Home{}, "Home"}
                    }
                ul {class: "flex space-x-4",

                    li { Link { to: Route::Home {}, class: "hover:text-green-400", "Home" } }

                    match has_token.read().as_ref(){
                        Some(Err(_e)) =>rsx!{li {Link {to:Route::LoginPage {  }, "Login"}} },
                        _ => rsx!{li {Link {to:Route::ShufflePage{  }, "Shuffle"}}}
                    }
                }
            }
        }
        main { class: "flex-grow container mx-auto p-4",
            SuspenseBoundary { 
                fallback: |_| rsx!{LoadingSpinner{}},
                Outlet::<Route>{}
            }
            
        }
        Footer {  }
    }
}

#[component]
pub fn Footer() -> Element {
    rsx! {
        footer {
            class: "bg-gray-800 text-gray-400 p-4 text-center mt-auto", // mt-auto helps if content is short
            p {
                "© 2025 BetterdSpotify - Created by "
                a { href: "https://hammamiomar.xyz", target: "_blank", rel: "noopener noreferrer", class: "text-green-400 hover:underline", "Omar Hammami" }
            }
        }
    }

    
}

#[component]
fn LoadingSpinner() -> Element {
    rsx! {
        div {
            class: "flex flex-col items-center justify-center p-8 text-center", // Centered
            // You can use an actual SVG spinner or a simple animation
            div {
                class: "animate-spin rounded-full h-16 w-16 border-t-4 border-b-4 border-green-500",
            }
            p {
                class: "mt-4 text-xl text-gray-300",
                "Fetching..."
            }
        }
    }
}