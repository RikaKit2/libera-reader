#![allow(non_snake_case)]
use dioxus::prelude::*;
use manganis;

mod router;
mod ui;
mod app;
use app::App;
const _TAILWIND_URL: &str = manganis::mg!(file("assets/tailwind.css"));

fn main() {
  let config = dioxus::desktop::Config::new()
    .with_custom_head(r#"<link rel="stylesheet" href="side_bar.css">"#.to_string());
  LaunchBuilder::desktop().with_cfg(config).launch(App);
}
