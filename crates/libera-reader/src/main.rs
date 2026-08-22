// Conditional global allocator: dhat::Alloc when profiling, mimalloc otherwise.
// Build with `cargo run --release --features dhat-heap` to get a `dhat-heap.json`
// profile on exit. See documentation/ram.md §3 / §12.
#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

#[cfg(not(feature = "dhat-heap"))]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use anyhow::Result;
use gpui::AppContext;
use gpui::{App, Bounds, Entity, TitlebarOptions, Window, WindowBounds, WindowOptions, px, size};
use gpui_component::Root;
use libera_reader::TOKIO;
use libera_reader::app_utils::{set_lang, start_services};
use libera_reader::ctx::Ctx;
use libera_reader::theme::init_theme;
use libera_reader::ui::{assets::Assets, pages::Pages};
use std::path::PathBuf;
use tokio::runtime::Runtime;
use utils::create_subscriber;

fn build_root_window(window: &mut Window, cx: &mut App) -> Entity<Root> {
  let setup_is_done = Ctx::global(cx).settings.read().setup_is_done;
  if setup_is_done {
    start_services(cx);
  }

  let ctx = Ctx::global(cx);
  let thumbnails_dir = ctx.app_dirs.read().thumbnails_dir.clone();
  let db = ctx.db.clone();

  let event_rx = ctx.event_tx.subscribe();
  let pages = Pages::new(window, cx, thumbnails_dir, &db, event_rx);
  cx.new(|cx| Root::new(pages, window, cx))
}

fn main() -> Result<()> {
  better_panic::install();
  create_subscriber()?;

  #[cfg(feature = "dhat-heap")]
  let _profiler = dhat::Profiler::new_heap();

  TOKIO.set(Runtime::new().unwrap()).expect("Failed to initialize Tokio runtime");

  let project_root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  let themes_dir = project_root_dir.join("themes");

  let app = gpui_platform::application().with_assets(Assets);
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
