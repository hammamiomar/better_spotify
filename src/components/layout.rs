use dioxus::prelude::*;
use crate::Route;

type AuthState = Signal<bool>;
#[component]
pub fn NavBar() -> Element {
    let is_logged_in = use_context::<AuthState>();

    rsx! {
        div {
            ul {
                li{ Link {to:Route::Home{}, "Home"}}
                if !is_logged_in(){
                    li {Link {to:Route::LoginPage {  }, "Login"}}
                } else{
                    li {"Logged in"}
                }
            }
        }
        Outlet::<Route>{}
        Footer {  }
    }
}

#[component]
pub fn Footer() -> Element {
    rsx! {
        footer {  
            a { href:"https://hammamiomar.xyz", "Personal Site"}
        }
    }
}