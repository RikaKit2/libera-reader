#![forbid(unsafe_code)]
mod theme;
mod ui;

use crate::ui::pages::Pages;
use crate::{theme::init_theme, ui::assets::Assets};
use anyhow::Result;
use gpui::AppContext;
use gpui::{App, Application, Bounds, Entity, TitlebarOptions, Window, WindowBounds, WindowOptions, px, size};
use gpui_component::Root;
use libera_reader_core::ctx::Ctx;
use mimalloc::MiMalloc;
use std::path::PathBuf;
use utils::{create_subscriber, debug};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn build_root_window(window: &mut Window, cx: &mut App) -> Entity<Root> {
  let pages = Pages::new(window, cx);
  cx.new(|cx| Root::new(pages, window, cx))
}

fn main() -> Result<()> {
  better_panic::install();
  create_subscriber()?;
  let project_root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  let path_to_assets = project_root_dir.join("assets").join("icons");
  let themes_dir = project_root_dir.join("themes");
  debug!("Themes dir exists: {}", themes_dir.exists());
  let assets = Assets::new(path_to_assets.clone());

  let app = Application::new().with_assets(assets);
  app.run(move |cx| {
    Ctx::init(cx);
    gpui_component::init(cx);
    init_theme(themes_dir, cx);

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
