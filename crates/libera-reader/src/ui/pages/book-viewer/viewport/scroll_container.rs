use crate::ui::pages::book_viewer::cache::{
  BookViewerCache, PageImageState, PageLinksState, PageTextState,
};
use crate::ui::pages::book_viewer::constants::viewport::{
  CONTAINER_PADDING, PAGE_GAP_Y, PREFETCH_DISTANCE,
};
use crate::ui::pages::book_viewer::loader::PageLoadRequest;
use crate::ui::pages::book_viewer::state::BookViewerState;
use crate::ui::pages::book_viewer::viewport::page::PageView;
use gpui::*;
use gpui_component::scroll::Scrollbar;
use gpui_component::{VirtualListScrollHandle, v_virtual_list};
use parking_lot::Mutex;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct ScrollContainer {
  state: Entity<BookViewerState>,
  cache: Arc<Mutex<BookViewerCache>>,
  scroll_handle: VirtualListScrollHandle,
  load_tx: tokio::sync::mpsc::UnboundedSender<PageLoadRequest>,
  visible_start: Arc<AtomicUsize>,
  visible_end: Arc<AtomicUsize>,
  last_rendered_page: usize,
  last_zoom_factor: f32,
  cached_item_sizes: Rc<Vec<Size<Pixels>>>,
  cached_page_offsets: Vec<Pixels>,
  cached_total_pages: usize,
  id: ElementId,
}

impl ScrollContainer {
  pub fn new(
    state: Entity<BookViewerState>, cache: Arc<Mutex<BookViewerCache>>,
    load_tx: tokio::sync::mpsc::UnboundedSender<PageLoadRequest>, visible_start: Arc<AtomicUsize>,
    visible_end: Arc<AtomicUsize>, cx: &mut App,
  ) -> Entity<Self> {
    let scroll_handle = VirtualListScrollHandle::new();
    let id = ElementId::Name("book-viewer-scroll-list".into());

    cx.new(|cx| {
      cx.observe(&state, |_, _, cx| cx.notify()).detach();
      Self {
        state,
        cache,
        scroll_handle,
        load_tx,
        visible_start,
        visible_end,
        last_rendered_page: 0,
        last_zoom_factor: 1.0,
        cached_item_sizes: Rc::new(Vec::new()),
        cached_page_offsets: Vec::new(),
        cached_total_pages: 0,
        id,
      }
    })
  }

  pub fn scroll_to_page(&mut self, page: usize) {
    if page > 0 {
      self.last_rendered_page = 0; // Trigger programmatic scroll in next render
    }
  }

  fn rebuild_geometry_if_needed(&mut self, total_items: usize, zoom_factor: f32, cx: &App) {
    if self.cached_total_pages == total_items
      && (self.last_zoom_factor - zoom_factor).abs() <= 0.0005
      && !self.cached_item_sizes.is_empty()
    {
      return;
    }
    let state_read = self.state.read(cx);
    let mut sizes = Vec::with_capacity(total_items);
    let mut offsets = Vec::with_capacity(total_items + 1);
    let mut acc_y = px(0.0);
    offsets.push(acc_y);

    for p in 1..=total_items {
      let dims = state_read.page_size(p);
      let w = dims.width * zoom_factor;
      let h = dims.height * zoom_factor + f32::from(PAGE_GAP_Y) + 32.0;
      let sz = size(px(w), px(h));
      sizes.push(sz);
      acc_y += sz.height;
      offsets.push(acc_y);
    }

    self.cached_item_sizes = Rc::new(sizes);
    self.cached_page_offsets = offsets;
    self.cached_total_pages = total_items;
  }

  fn page_at_scroll_y(&self, scroll_y: Pixels) -> usize {
    if self.cached_page_offsets.len() <= 1 {
      return 1;
    }
    let probe_y = scroll_y + px(80.0);
    match self
      .cached_page_offsets
      .binary_search_by(|offset| offset.partial_cmp(&probe_y).unwrap_or(std::cmp::Ordering::Equal))
    {
      Ok(idx) => (idx + 1).clamp(1, self.cached_total_pages),
      Err(idx) => idx.clamp(1, self.cached_total_pages),
    }
  }
}

impl Render for ScrollContainer {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    self.cache.lock().flush_evictions(cx);
    let (doc_loaded, total_pages, current_book, book_size, book_hash, zoom_factor, active_page) = {
      let s = self.state.read(cx);
      (
        s.current_document.is_some(),
        s.total_pages,
        s.current_book.clone(),
        s.book_size,
        s.book_hash.clone(),
        s.zoom_factor,
        s.current_page,
      )
    };
    if !doc_loaded || book_size == 0 {
      return div().size_full().p(CONTAINER_PADDING);
    }

    let total_items = total_pages.max(1);
    let zoom_changed = (zoom_factor - self.last_zoom_factor).abs() > 0.0005;

    let old_fraction = if zoom_changed && !self.cached_page_offsets.is_empty() {
      let current_scroll_y = (-self.scroll_handle.offset().y).max(px(0.0));
      let idx = (active_page.saturating_sub(1)).min(self.cached_item_sizes.len().saturating_sub(1));
      let old_y = self.cached_page_offsets.get(idx).copied().unwrap_or(px(0.0));
      let old_h = self.cached_item_sizes.get(idx).map(|s| s.height).unwrap_or(px(1.0));
      if old_h > px(0.0) { ((current_scroll_y - old_y) / old_h).clamp(0.0, 1.0) } else { 0.0 }
    } else {
      0.0
    };

    self.rebuild_geometry_if_needed(total_items, zoom_factor, cx);
    let item_sizes = self.cached_item_sizes.clone();

    let page_changed_programmatically =
      (active_page != self.last_rendered_page) && (active_page > 0) && (active_page <= total_items);

    if zoom_changed {
      self.last_zoom_factor = zoom_factor;
      self.last_rendered_page = active_page;
      let idx = active_page.saturating_sub(1);
      let new_page_y = self.cached_page_offsets.get(idx).copied().unwrap_or(px(0.0));
      let new_page_h = self.cached_item_sizes.get(idx).map(|s| s.height).unwrap_or(px(0.0));
      let target_y = new_page_y + new_page_h * old_fraction;
      self.scroll_handle.set_offset(point(px(0.0), -target_y));
    } else if page_changed_programmatically {
      self.last_rendered_page = active_page;
      let idx = active_page.saturating_sub(1);
      let target_y = self.cached_page_offsets.get(idx).copied().unwrap_or(px(0.0));
      self.scroll_handle.set_offset(point(px(0.0), -target_y));
    }

    let suppress_scroll_sync = zoom_changed || page_changed_programmatically;
    let scroll = self.scroll_handle.clone();
    let state_entity = self.state.clone();
    let cache_arc = self.cache.clone();
    let load_tx = self.load_tx.clone();
    let book_path_opt = current_book.map(|b| b.as_pathbuf());

    let list = v_virtual_list(
      cx.entity().clone(),
      self.id.clone(),
      item_sizes,
      move |view: &mut ScrollContainer, visible_range, window, cx| {
        let start_page_1based = (visible_range.start + 1).min(total_pages.max(1));
        let end_page_1based = visible_range.end.min(total_pages.max(1)).max(start_page_1based);
        view.visible_start.store(start_page_1based, Ordering::Relaxed);
        view.visible_end.store(end_page_1based, Ordering::Relaxed);

        if !suppress_scroll_sync {
          let current_scroll_y = (-view.scroll_handle.offset().y).max(px(0.0));
          let detected_page = view.page_at_scroll_y(current_scroll_y);
          if detected_page != view.last_rendered_page && detected_page <= total_pages {
            view.last_rendered_page = detected_page;
            let state_for_defer = state_entity.clone();
            window.defer(cx, move |_window, cx| {
              state_for_defer.update(cx, |s, cx| {
                if s.current_page != detected_page {
                  s.current_page = detected_page;
                  cx.notify();
                }
              });
            });
          }
        }

        let mut page_elements = Vec::with_capacity(visible_range.len());

        if let Some(book_path) = &book_path_opt {
          let mut cache_lock = cache_arc.lock();
          cache_lock.ensure_min_capacity(visible_range.len() + PREFETCH_DISTANCE * 2 + 4);

          let target_dpi = crate::ui::pages::book_viewer::constants::viewport::PAGE_RENDER_DPI;

          for page_idx in visible_range {
            let page_num = page_idx + 1;
            if page_num > total_pages {
              break;
            }

            let img_state = cache_lock.get_image(page_num);
            let (img, is_img_loading) = match img_state {
              Some(PageImageState::Loaded(img)) => (Some(img), false),
              Some(PageImageState::Loading) => (None, true),
              Some(PageImageState::Failed(_)) => (None, false),
              Some(PageImageState::Unloaded) | None => {
                cache_lock.insert_image(page_num, PageImageState::Loading);
                cache_lock.set_loading_dpi(page_num, target_dpi);
                let _ = load_tx.send(PageLoadRequest {
                  page: page_num,
                  book_path: book_path.clone(),
                  dpi: target_dpi,
                  book_size,
                  book_hash: book_hash.clone(),
                });
                (None, true)
              }
            };

            let stext = match cache_lock.get_text(page_num) {
              Some(PageTextState::Loaded(t)) => Some(t.clone()),
              _ => None,
            };

            let links = match cache_lock.get_links(page_num) {
              Some(PageLinksState::Loaded(l)) => Some(l.clone()),
              _ => None,
            };

            let page_view =
              PageView::new(page_num, state_entity.clone(), img, is_img_loading, stext, links);

            page_elements.push(
              div()
                .w_full()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .py(PAGE_GAP_Y / 2.0)
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
                .child(page_view),
            );
          }

          // Prefetch adjacent pages into disk cache & memory
          let prefetch_start = start_page_1based.saturating_sub(1).max(1);
          let prefetch_end = (end_page_1based + PREFETCH_DISTANCE).min(total_pages);
          for pf_page in prefetch_start..=prefetch_end {
            if pf_page >= start_page_1based && pf_page <= end_page_1based {
              continue;
            }
            if cache_lock.get_image(pf_page).is_none()
              && cache_lock.get_loading_dpi(pf_page).is_none()
            {
              cache_lock.insert_image(pf_page, PageImageState::Loading);
              cache_lock.set_loading_dpi(pf_page, target_dpi);
              let _ = load_tx.send(PageLoadRequest {
                page: pf_page,
                book_path: book_path.clone(),
                dpi: target_dpi,
                book_size,
                book_hash: book_hash.clone(),
              });
            }
          }
        }

        page_elements
      },
    )
    .track_scroll(&scroll)
    .size_full();

    div()
      .relative()
      .size_full()
      .p(CONTAINER_PADDING)
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
      .child(list)
      .child(
        div()
          .absolute()
          .top_0()
          .right_0()
          .bottom_0()
          .w_2()
          .child(Scrollbar::new(&self.scroll_handle)),
      )
  }
}
