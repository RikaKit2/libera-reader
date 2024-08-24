use crate::router::Route;
use dioxus::prelude::*;
pub mod pages;

#[component]
pub fn Root() -> Element {
    rsx! {
        Outlet::<Route> {}
    }
}

#[component]
pub fn PageNotFound(route: Vec<String>) -> Element {
    rsx! {
        h1 { "Page not found" }
        p { "We are terribly sorry, but the page you requested doesn't exist." }
        pre { color: "red", "log:\nattemped to navigate to: {route:?}" }
    }
}
