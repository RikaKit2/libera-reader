mod ui;

use crate::ui::assets::Assets;
use crate::ui::pages::Pages;
use gpui::{prelude::*, px, size, App, Application, Bounds, Entity, TitlebarOptions, Window, WindowBounds, WindowOptions};
use std::path::PathBuf;
use tracing::Level;

fn build_root_window(_window: &mut Window, cx: &mut App) -> Entity<Pages> {
  cx.new(|_| Pages::new())
}

fn main() {
  let subscriber = tracing_subscriber::fmt()
    .pretty()
    .without_time()
    .compact()
    .with_file(false)
    .with_line_number(false)
    .with_thread_ids(true)
    .with_target(false)
    .with_max_level(Level::DEBUG)
    .finish();
  tracing::subscriber::set_global_default(subscriber).unwrap();
  let path_to_assets = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets").join("icons");
  Application::new()
    .with_assets(Assets::new(path_to_assets))
    .run(|cx: &mut App| {
      let bounds = Bounds::centered(None, size(px(700.0), px(400.0)), cx);
      let window_options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        titlebar: Some(TitlebarOptions { title: Some("Libera Reader".into()), ..Default::default() }),
        ..Default::default()
      };
      cx.open_window(window_options, build_root_window).unwrap();
    });
}
