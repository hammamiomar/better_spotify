#![allow(non_snake_case)]
use dioxus::prelude::*;

mod components;
mod routes;

#[cfg(feature = "server")]
mod server;
#[cfg(feature = "server")]
mod auth;

use crate::components::layout::*;
use crate::routes::pages::*;

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
      Router::<Route>{ }
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
    #[route("/callback")]
    CallBack{},
}
#[component]
fn CallBack() -> Element{
    todo!()
}

