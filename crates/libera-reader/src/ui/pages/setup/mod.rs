use crate::router::Route;
use dioxus::prelude::*;

#[component]
pub fn Setup() -> Element {
    let setup_ready: bool = true;
    let nav = navigator();
    if setup_ready {
        nav.push(Route::Library {});
    };

    rsx! {
        div { "setup" }
    }
}
