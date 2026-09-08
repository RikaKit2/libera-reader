use crate::app_ext::AppExt;
use crate::ui::pages::book_viewer::cache::{
  BookViewerCache, PageImageState, PageLinksState, PageTextState,
};
use crate::ui::pages::book_viewer::constants::viewport::PAGE_RENDER_DPI;
use crate::ui::pages::book_viewer::loader::{PageLoadRequest, spawn_book_page_loader};
use crate::ui::pages::book_viewer::state::BookViewerState;
use crate::ui::pages::book_viewer::viewport::page::PageView;
use gpui::*;
use parking_lot::Mutex;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct PagedContainer {
  state: Entity<BookViewerState>,
  cache: Arc<Mutex<BookViewerCache>>,
  load_tx: tokio::sync::mpsc::UnboundedSender<PageLoadRequest>,
  visible_start: Arc<AtomicUsize>,
  visible_end: Arc<AtomicUsize>,
}

impl PagedContainer {
  pub fn new(state: Entity<BookViewerState>, cx: &mut App) -> Entity<Self> {
    let cache = Arc::new(Mutex::new(BookViewerCache::new()));
    let visible_start = Arc::new(AtomicUsize::new(1));
    let visible_end = Arc::new(AtomicUsize::new(1));

    let (load_tx, load_rx) = tokio::sync::mpsc::unbounded_channel::<PageLoadRequest>();
    let (notify_tx, mut notify_rx) = tokio::sync::mpsc::unbounded_channel::<()>();

    let runtime = crate::TOKIO.get().unwrap();
    let coordinator = cx.services().extraction_coordinator.clone();

    spawn_book_page_loader(
      runtime,
      cache.clone(),
      visible_start.clone(),
      visible_end.clone(),
      load_rx,
      notify_tx,
      coordinator,
    );

    cx.new(|cx| {
      cx.spawn(|this: gpui::WeakEntity<Self>, cx: &mut AsyncApp| {
        let mut owned_cx = cx.clone();
        async move {
          while notify_rx.recv().await.is_some() {
            while notify_rx.try_recv().is_ok() {}
            let _ = this.update(&mut owned_cx, |_, cx| cx.notify());
          }
        }
      })
      .detach();

      Self { state, cache, load_tx, visible_start, visible_end }
    })
  }
}

impl Render for PagedContainer {
  fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let (current_page, current_book) = {
      let s = self.state.read(cx);
      (s.current_page, s.current_book.clone())
    };

    // Update active visible page window in atomic for prefetch bounds
    self.visible_start.store(current_page, Ordering::Relaxed);
    self.visible_end.store(current_page, Ordering::Relaxed);

    let (img, is_img_loading, stext, links) = match current_book {
      Some(book) => {
        let path = book.as_pathbuf();
        let mut cache_lock = self.cache.lock();

        let (img, is_img_loading) = match cache_lock.get_image(current_page) {
          Some(PageImageState::Loaded(img)) => (Some(img.clone()), false),
          Some(PageImageState::Loading) => (None, true),
          Some(PageImageState::Failed(_)) => (None, false),
          Some(PageImageState::Unloaded) | None => {
            cache_lock.insert_image(current_page, PageImageState::Loading);
            let _ = self.load_tx.send(PageLoadRequest {
              page: current_page,
              book_path: path,
              dpi: PAGE_RENDER_DPI,
            });
            (None, true)
          }
        };

        let stext = match cache_lock.get_text(current_page) {
          Some(PageTextState::Loaded(t)) => Some(t.clone()),
          _ => None,
        };

        let links = match cache_lock.get_links(current_page) {
          Some(PageLinksState::Loaded(l)) => Some(l.clone()),
          _ => None,
        };

        (img, is_img_loading, stext, links)
      }
      None => (None, false, None, None),
    };

    let selection_handle =
      self.state.update(cx, |s, cx| s.get_or_create_selection_handle(current_page, window, cx));

    let page_view = PageView::new(
      current_page,
      self.state.clone(),
      img,
      is_img_loading,
      stext,
      links,
      selection_handle,
    );

    div().size_full().flex().flex_col().items_center().justify_center().p_4().child(page_view)
  }
}
