use crate::ui::pages::book_viewer::cache::{
  BookViewerCache, PageImageState, PageLinksState, PageTextState,
};
use crate::ui::pages::book_viewer::loader::PageLoadRequest;
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
  pub fn new(
    state: Entity<BookViewerState>, cache: Arc<Mutex<BookViewerCache>>,
    load_tx: tokio::sync::mpsc::UnboundedSender<PageLoadRequest>, visible_start: Arc<AtomicUsize>,
    visible_end: Arc<AtomicUsize>, cx: &mut App,
  ) -> Entity<Self> {
    cx.new(|_cx| Self { state, cache, load_tx, visible_start, visible_end })
  }
}

impl Render for PagedContainer {
  fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    self.cache.lock().flush_evictions(cx);
    let (current_page, current_book, book_size, book_hash, zoom_factor) = {
      let s = self.state.read(cx);
      (s.current_page, s.current_book.clone(), s.book_size, s.book_hash.clone(), s.zoom_factor)
    };

    // Update active visible page window in atomic for prefetch bounds
    self.visible_start.store(current_page, Ordering::Relaxed);
    self.visible_end.store(current_page, Ordering::Relaxed);

    let (img, is_img_loading, stext, links) = match current_book {
      Some(book) => {
        let path = book.as_pathbuf();
        let mut cache_lock = self.cache.lock();

        let scale_factor = window.scale_factor();
        let target_dpi =
          crate::ui::pages::book_viewer::loader::compute_target_dpi(zoom_factor, scale_factor);
        let cached_dpi = cache_lock.get_image_dpi(current_page);
        let is_stale_dpi = cached_dpi.is_some_and(|d| d != target_dpi);

        let img_state = cache_lock.get_image(current_page).cloned();
        let (img, is_img_loading) = match img_state {
          Some(PageImageState::Loaded(img)) if !is_stale_dpi => (Some(img), false),
          Some(PageImageState::Loaded(img)) => {
            if cache_lock.get_loading_dpi(current_page) != Some(target_dpi) {
              cache_lock.set_loading_dpi(current_page, target_dpi);
              let _ = self.load_tx.send(PageLoadRequest {
                page: current_page,
                book_path: path.clone(),
                dpi: target_dpi,
                book_size,
                book_hash: book_hash.clone(),
              });
            }
            (Some(img), false)
          }
          Some(PageImageState::Loading) => (None, true),
          Some(PageImageState::Failed(_)) => (None, false),
          Some(PageImageState::Unloaded) | None => {
            cache_lock.insert_image(current_page, PageImageState::Loading);
            cache_lock.set_loading_dpi(current_page, target_dpi);
            let _ = self.load_tx.send(PageLoadRequest {
              page: current_page,
              book_path: path,
              dpi: target_dpi,
              book_size,
              book_hash,
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

    div()
      .size_full()
      .flex()
      .flex_col()
      .items_center()
      .justify_center()
      .p_4()
      .on_scroll_wheel(cx.listener(|this, event: &ScrollWheelEvent, window, cx| {
        if event.modifiers.control || event.modifiers.platform {
          cx.stop_propagation();
          let delta = event.delta.pixel_delta(px(20.0)).y;
          let step = if delta > px(0.0) { 0.10 } else { -0.10 };
          gpui_base::TextSelection::clear(window, cx);
          this.state.update(cx, |s, cx| {
            s.clear_selection_handles();
            s.adjust_zoom_by(step);
            cx.notify();
          });
        }
      }))
      .child(page_view)
  }
}
