mod app_root;
mod cache;
mod image_utils;
mod loader;
mod pages;

use app_root::AppRoot;
use gpui::prelude::*;
use gpui::{Bounds, WindowBounds, WindowOptions, px, size};
use mimalloc::MiMalloc;
use std::sync::OnceLock;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

pub static TOKIO: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

fn main() {
  let runtime = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
  TOKIO.set(runtime).ok();

  // ⚡️ MIGRATION: Application initialization now goes through gpui_platform
  let app = gpui_platform::application().with_assets(gpui_component_assets::Assets);
  app.run(move |cx| {
    gpui_component::init(cx);
    let bounds = Bounds::centered(None, size(px(1100.0), px(750.0)), cx);
    cx.open_window(
      WindowOptions { window_bounds: Some(WindowBounds::Windowed(bounds)), ..Default::default() },
      |window, cx| {
        let view = cx.new(AppRoot::new);
        cx.new(|cx| gpui_component::Root::new(view, window, cx))
      },
    )
    .expect("Failed to open window");
  });
}
