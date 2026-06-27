#![forbid(unsafe_code)]

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;
rust_i18n::i18n!("../../locales");

mod app_utils;
mod books_state;
mod theme;
mod ui;

use crate::app_utils::{set_lang, start_services};
use crate::theme::init_theme;
use crate::ui::{assets::Assets, pages::Pages};
use anyhow::Result;
use gpui::AppContext;
use gpui::{
  App, Application, Bounds, Entity, TitlebarOptions, Window, WindowBounds, WindowOptions, px, size,
};
use gpui_component::Root;
use libera_reader_core::ctx::Ctx;
use mimalloc::MiMalloc;
use std::path::PathBuf;
use std::sync::OnceLock;
use tokio::runtime::Runtime;
use utils::create_subscriber;

pub static TOKIO: OnceLock<Runtime> = OnceLock::new();

fn build_root_window(window: &mut Window, cx: &mut App) -> Entity<Root> {
  let setup_is_done = Ctx::global(cx).settings.read().setup_is_done;
  if setup_is_done {
    start_services(cx);
  }

  let ctx = Ctx::global(cx);
  let initial_books = ctx.db.scan_all_books().unwrap_or_default();

  let event_rx = ctx.event_tx.subscribe();
  let pages = Pages::new(window, cx, initial_books, event_rx);
  cx.new(|cx| Root::new(pages, window, cx))
}

fn main() -> Result<()> {
  better_panic::install();
  create_subscriber()?;

  TOKIO.set(Runtime::new().unwrap()).expect("Failed to initialize Tokio runtime");

  let project_root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  let themes_dir = project_root_dir.join("themes");

  let app = Application::new().with_assets(Assets);
  app.run(move |cx| {
    Ctx::init(cx);
    gpui_component::init(cx);
    init_theme(themes_dir, cx);
    set_lang(cx);

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
