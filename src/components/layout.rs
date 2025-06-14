use dioxus::{document::eval, prelude::*};
use crate::{api::{check_auth, logout}, Route};

#[component]
pub fn NavBar() -> Element {
    let is_authenticated = use_server_future( || async {
        check_auth().await})?;

    rsx! {
        header {
            class: "bg-gray-800 text-white p-4 shadow-md",
            nav {
                class: "container mx-auto flex justify-between items-center",

                div{class: "text-xl font-bold hover:text-green-400",
                     Link {to:Route::Home{}, "BetterdSpotify"}
                    }
                ul {class: "flex space-x-4",

                    li { Link { to: Route::Home {}, class: "hover:text-green-400", "Home" } }

                    match is_authenticated.read().as_ref(){
                        None => rsx!{}, // Still loading, show nothing
                        Some(Err(_e)) => rsx!{li {Link {to:Route::LoginPage {  }, "Login"}} },
                        Some(Ok(false)) => rsx!{li {Link {to:Route::LoginPage {  }, "Login"}} },
                        Some(Ok(true)) => rsx!{
                            li {Link {to:Route::ShufflePage{  }, "Shuffle"}}

                            li {
                                button {
                                    class: "bg-red-600 hover:bg-red-700 text-white font-bold py-1 px-3 rounded transition-colors",
                                    onclick: move |_| async move {
                                        let _ = logout().await;
                                        let _ = eval(r#"window.location.href = "/auth/logout";"#);
                                    },
                                    "Logout"
                                }
                        
                            }
                        }
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