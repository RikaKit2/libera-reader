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
use libera_reader::app_dirs::AppDirs;
use libera_reader::app_ext::AppExt;
use libera_reader::books_state::{BooksState, BooksStateEntity};
use libera_reader::db::DB;
use libera_reader::not_cached_books::NotCachedBooks;
use libera_reader::services::{Services, start_services};
use libera_reader::settings::{SETTINGS, apply_language};
use libera_reader::theme::init_theme;
use libera_reader::ui::{assets::Assets, pages::Pages};
use std::path::PathBuf;
use tokio::runtime::Runtime;
use utils::create_subscriber;

fn build_root_window(window: &mut Window, cx: &mut App) -> Entity<Root> {
  let books_state = cx.books_state().clone();
  let books_state_entity = cx.new(|cx| books_state.attach_ui(cx));
  cx.set_global(BooksStateEntity(books_state_entity));

  if cx.settings().read().setup_is_done {
    start_services(cx);
  }
  let pages = Pages::new(window, cx);
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
    let app_dirs = AppDirs::new_with_default_data_dir().unwrap();
    let path_to_db = app_dirs.read().path_to_db.clone();
    let db = DB::new(path_to_db).unwrap();
    let settings = SETTINGS::new(db.clone()).unwrap();
    let not_cached_books = NotCachedBooks::new();
    let books_state = BooksState::new(app_dirs.read().thumbnails_dir.clone(), &db);

    cx.set_global(app_dirs);
    cx.set_global(db);
    cx.set_global(settings);
    cx.set_global(not_cached_books);
    cx.set_global(books_state);

    let services = Services::new(cx).unwrap();
    cx.set_global(services);
    gpui_component::init(cx);
    init_theme(themes_dir, cx);
    apply_language(cx);
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
