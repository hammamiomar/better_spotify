#![allow(non_snake_case)]
use dioxus::prelude::*;

mod components;
mod routes;

#[cfg(feature = "server")]
mod server;
#[cfg(feature = "server")]
mod auth;
pub mod api;
pub mod api_models;

use crate::components::layout::*;
use crate::routes::pages::*;

static CSS: Asset = asset!("/assets/tailwind.css");

// The entry point for the server
#[cfg(feature = "server")]
#[tokio::main]
async fn main() {
    if let Err(e) = server::start_server().await{
        eprintln!("Server failed to start: {}", e);
        std::process::exit(1);
    }
}
// For any other platform, we just launch the app
#[cfg(not(feature = "server"))]
fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    use_context_provider(|| Signal::new(false));
    
    rsx! {
        document::Stylesheet { href:CSS}
        div{ class:"min-h-screen bg-gray-900 text-gray-100 flex flex-col",
            Router::<Route>{ }
        }
    }
}

//Router
#[derive(Routable, Clone, PartialEq)]
enum Route {
    #[layout(NavBar)]
    #[route("/")]
    Home {},
    #[route("/login")]
    LoginPage {},
    #[route("/shuffle")]
    ShufflePage{},
    #[route("/callback")]
    CallBack{},
}
#[component]
fn CallBack() -> Element{
    todo!()
}

