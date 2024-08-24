#![allow(non_snake_case)]
use crate::router::Route;
use dioxus::prelude::*;
use manganis;

mod router;
mod ui;


const _TAILWIND_URL: &str = manganis::mg!(file("assets/tailwind.css"));

fn main() {
    launch(|| {
        rsx! {
            Router::<Route> {}
        }
    });
}
