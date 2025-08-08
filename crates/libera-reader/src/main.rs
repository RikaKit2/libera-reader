mod ui;

use crate::ui::assets::Assets;
use crate::ui::pages::Pages;
use anyhow::Result;
use gpui::{px, size, App, Application, Bounds, Entity, TitlebarOptions, Window, WindowBounds, WindowOptions};
use libera_reader_core::ctx::Ctx;
use std::path::PathBuf;
use utils::create_subscriber;

fn build_root_window(_window: &mut Window, cx: &mut App) -> Entity<Pages> { Pages::new(cx) }

fn main() -> Result<()> {
  create_subscriber()?;
  let path_to_assets = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets").join("icons");
  Application::new().with_assets(Assets::new(path_to_assets)).run(|cx: &mut App| {
    Ctx::init(cx);
    let bounds = Bounds::centered(None, size(px(700.0), px(400.0)), cx);
    let window_options = WindowOptions {
      window_bounds: Some(WindowBounds::Windowed(bounds)),
      titlebar: Some(TitlebarOptions { title: Some("Libera Reader".into()), ..Default::default() }),
      ..Default::default()
    };
    cx.open_window(window_options, build_root_window).unwrap();
  });
  Ok(())
}
