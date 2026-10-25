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
    cx.new(|cx| {
      cx.observe(&state, |_, _, cx| cx.notify()).detach();
      Self { state, cache, load_tx, visible_start, visible_end }
    })
  }
}

impl Render for PagedContainer {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    self.cache.lock().flush_evictions(cx);
    let (current_page, current_book, book_size, book_hash) = {
      let s = self.state.read(cx);
      (s.current_page, s.current_book.clone(), s.book_size, s.book_hash.clone())
    };

    // Update active visible page window in atomic for prefetch bounds
    self.visible_start.store(current_page, Ordering::Relaxed);
    self.visible_end.store(current_page, Ordering::Relaxed);

    let (img, is_img_loading, stext, links) = match current_book {
      Some(book) => {
        let path = book.as_pathbuf();
        let mut cache_lock = self.cache.lock();

        let target_dpi = crate::ui::pages::book_viewer::constants::viewport::PAGE_RENDER_DPI;
        let img_state = cache_lock.get_image(current_page);
        let (img, is_img_loading) = match img_state {
          Some(PageImageState::Loaded(img)) => (Some(img), false),
          Some(PageImageState::Loading) => (None, true),
          Some(PageImageState::Failed(_)) => (None, false),
          Some(PageImageState::Unloaded) | None => {
            cache_lock.insert_image(current_page, PageImageState::Loading);
            cache_lock.set_loading_dpi(current_page, target_dpi);
            let _ = self.load_tx.send(PageLoadRequest {
              page: current_page,
              book_path: path.clone(),
              dpi: target_dpi,
              book_size,
              book_hash: book_hash.clone(),
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
        let total_pages = self.state.read(cx).total_pages;
        let prefetch_start = current_page.saturating_sub(1).max(1);
        let prefetch_end = (current_page
          + crate::ui::pages::book_viewer::constants::viewport::PREFETCH_DISTANCE)
          .min(total_pages);
        for pf_page in prefetch_start..=prefetch_end {
          if pf_page == current_page {
            continue;
          }
          if cache_lock.get_image(pf_page).is_none()
            && cache_lock.get_loading_dpi(pf_page).is_none()
          {
            cache_lock.insert_image(pf_page, PageImageState::Loading);
            cache_lock.set_loading_dpi(pf_page, target_dpi);
            let _ = self.load_tx.send(PageLoadRequest {
              page: pf_page,
              book_path: path.clone(),
              dpi: target_dpi,
              book_size,
              book_hash: book_hash.clone(),
            });
          }
        }

        (img, is_img_loading, stext, links)
      }
      None => (None, false, None, None),
    };

    let page_view =
      PageView::new(current_page, self.state.clone(), img, is_img_loading, stext, links);

    div()
      .size_full()
      .flex()
      .flex_col()
      .items_center()
      .justify_center()
      .p_4()
      .on_scroll_wheel(cx.listener(|this, event: &ScrollWheelEvent, _window, cx| {
        if event.modifiers.control || event.modifiers.platform {
          cx.stop_propagation();
          let delta = event.delta.pixel_delta(px(20.0)).y;
          let step = if delta > px(0.0) { 0.10 } else { -0.10 };
          this.state.update(cx, |s, cx| {
            s.adjust_zoom_by(step);
            cx.notify();
          });
        }
      }))
      .child(page_view)
  }
}
