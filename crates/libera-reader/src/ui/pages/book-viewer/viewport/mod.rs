pub mod page;
pub mod paged_container;
pub mod scroll_container;

pub use paged_container::PagedContainer;
pub use scroll_container::ScrollContainer;

use crate::app_ext::AppExt;
use crate::db::models::books::book::BookPath;
use crate::ui::pages::book_viewer::cache::BookViewerCache;
use crate::ui::pages::book_viewer::loader::{PageLoadRequest, spawn_book_page_loader};
use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::*;
use gpui_component::ActiveTheme;
use parking_lot::Mutex;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;

pub struct Viewport {
  state: Entity<BookViewerState>,
  cache: Arc<Mutex<BookViewerCache>>,
  scroll_container: Entity<ScrollContainer>,
  paged_container: Entity<PagedContainer>,
  last_book_path: Option<BookPath>,
}

impl Viewport {
  pub fn new(state: Entity<BookViewerState>, cx: &mut App) -> Entity<Self> {
    let cache = Arc::new(Mutex::new(BookViewerCache::new()));
    let visible_start = Arc::new(AtomicUsize::new(0));
    let visible_end = Arc::new(AtomicUsize::new(0));

    let (load_tx, load_rx) = tokio::sync::mpsc::unbounded_channel::<PageLoadRequest>();
    let (notify_tx, mut notify_rx) = tokio::sync::mpsc::unbounded_channel::<()>();

    let runtime = crate::TOKIO.get().unwrap();
    let coordinator = cx.services().extraction_coordinator.clone();
    let workers = cx.settings().read().workers_num.max(1) as usize;
    let app_dirs = cx.app_dirs().clone();

    spawn_book_page_loader(
      runtime,
      cache.clone(),
      visible_start.clone(),
      visible_end.clone(),
      load_rx,
      notify_tx,
      coordinator,
      app_dirs,
      workers,
    );

    let scroll_container = ScrollContainer::new(
      state.clone(),
      cache.clone(),
      load_tx.clone(),
      visible_start.clone(),
      visible_end.clone(),
      cx,
    );
    let paged_container =
      PagedContainer::new(state.clone(), cache.clone(), load_tx, visible_start, visible_end, cx);

    let last_book_path = state.read(cx).current_book.clone();

    cx.new(|cx| {
      cx.observe(&state, |_, _, cx| cx.notify()).detach();
      let sc = scroll_container.clone();
      let pc = paged_container.clone();
      cx.spawn(|this: gpui::WeakEntity<Self>, cx: &mut AsyncApp| {
        let mut owned_cx = cx.clone();
        async move {
          while notify_rx.recv().await.is_some() {
            while notify_rx.try_recv().is_ok() {}
            sc.update(&mut owned_cx, |_, cx| cx.notify());
            pc.update(&mut owned_cx, |_, cx| cx.notify());
            let _ = this.update(&mut owned_cx, |_, cx| cx.notify());
          }
        }
      })
      .detach();

      Self { state, cache, scroll_container, paged_container, last_book_path }
    })
  }

  pub fn scroll_to_page(&self, page: usize, cx: &mut App) {
    self.scroll_container.update(cx, |sc, _cx| {
      sc.scroll_to_page(page);
    });
  }
}

impl Render for Viewport {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let current_book = self.state.read(cx).current_book.clone();
    if current_book != self.last_book_path {
      self.last_book_path = current_book;
      let mut cache_lock = self.cache.lock();
      cache_lock.clear();
      cache_lock.flush_evictions(cx);
    }

    let is_continuous = self.state.read(cx).layout_mode.is_continuous();
    let bg = cx.theme().background;

    div().size_full().bg(bg).flex_1().overflow_hidden().child(match is_continuous {
      true => self.scroll_container.clone().into_any_element(),
      false => self.paged_container.clone().into_any_element(),
    })
  }
}
