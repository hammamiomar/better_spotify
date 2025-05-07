use dioxus::prelude::*;

#[component]
pub fn Home() -> Element {
    rsx! {
        h1 { "Better spotify because spotify is run via algos who just dont get it" }
        p{ "True RNG shuffle..."}
    }
}
type AuthState = Signal<bool>;
#[component]
pub fn LoginPage() -> Element {
    let is_logged_in = use_context::<AuthState>();
    if is_logged_in(){
        return rsx!{ p{"You are already logged in."}}
    }
    rsx! {
        h1{ "Login required"}
        p{"Please login with spotify"}
        a {href:"/login",
            button{"Login with Spotify"}}
    }
    
}