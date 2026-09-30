use crate::app_ext::AppExt;
use crate::ui::pages::book_viewer::cache::{
  BookViewerCache, PageImageState, PageLinksState, PageTextState,
};
use crate::ui::pages::book_viewer::constants::viewport::{CONTAINER_PADDING, PAGE_GAP_Y};
use crate::ui::pages::book_viewer::loader::{PageLoadRequest, spawn_book_page_loader};
use crate::ui::pages::book_viewer::state::BookViewerState;
use crate::ui::pages::book_viewer::viewport::page::PageView;
use gpui::*;
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
  id: ElementId,
}

impl ScrollContainer {
  pub fn new(state: Entity<BookViewerState>, cx: &mut App) -> Entity<Self> {
    let cache = Arc::new(Mutex::new(BookViewerCache::new()));
    let visible_start = Arc::new(AtomicUsize::new(0));
    let visible_end = Arc::new(AtomicUsize::new(0));
    let scroll_handle = VirtualListScrollHandle::new();
    let id = ElementId::Name("book-viewer-scroll-list".into());

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

      Self {
        state,
        cache,
        scroll_handle,
        load_tx,
        visible_start,
        visible_end,
        last_rendered_page: 0,
        last_zoom_factor: 1.0,
        id,
      }
    })
  }

  pub fn scroll_to_page(&mut self, page: usize) {
    if page > 0 {
      self.last_rendered_page = 0; // Trigger programmatic scroll in next render
    }
  }
}

impl Render for ScrollContainer {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let (total_pages, current_book, book_size, book_hash, zoom_factor, active_page) = {
      let s = self.state.read(cx);
      (
        s.total_pages,
        s.current_book.clone(),
        s.book_size,
        s.book_hash.clone(),
        s.zoom_factor,
        s.current_page,
      )
    };
    let total_items = total_pages.max(1);

    // Compute dynamic item sizes for each page based on its aspect ratio & zoom
    let item_sizes: Rc<Vec<gpui::Size<Pixels>>> = {
      let state_read = self.state.read(cx);
      Rc::new(
        (1..=total_items)
          .map(|p| {
            let dims = state_read.page_size(p);
            let w = dims.width * zoom_factor;
            let h = dims.height * zoom_factor + f32::from(PAGE_GAP_Y) + 32.0; // page + gap + badge
            size(px(w), px(h))
          })
          .collect(),
      )
    };

    let page_y_offset = |target_page: usize| -> Pixels {
      let mut y = px(0.0);
      for p in 1..target_page {
        if p <= item_sizes.len() {
          y += item_sizes[p - 1].height;
        }
      }
      y
    };

    // If zoom changed, anchor the scroll position smoothly to the current relative position within active page.
    // If active page changed externally (button click, outline click, input), scroll to it.
    let zoom_changed = (zoom_factor - self.last_zoom_factor).abs() > 0.001;
    let page_changed_programmatically =
      (active_page != self.last_rendered_page) && (active_page > 0) && (active_page <= total_items);

    if zoom_changed {
      let old_zoom = self.last_zoom_factor;
      self.last_zoom_factor = zoom_factor;
      self.last_rendered_page = active_page;

      let current_scroll_y = (-self.scroll_handle.offset().y).max(px(0.0));
      // Calculate where active_page was with the old zoom factor
      let mut old_page_y = px(0.0);
      let state_read = self.state.read(cx);
      for p in 1..active_page {
        let dims = state_read.page_size(p);
        let h = dims.height * old_zoom + f32::from(PAGE_GAP_Y) + 32.0;
        old_page_y += px(h);
      }
      let old_page_dims = state_read.page_size(active_page);
      let old_page_h = px(old_page_dims.height * old_zoom + f32::from(PAGE_GAP_Y) + 32.0);

      let fraction = if old_page_h > px(0.0) {
        ((current_scroll_y - old_page_y) / old_page_h).clamp(0.0, 1.0)
      } else {
        0.0
      };

      let new_page_y = page_y_offset(active_page);
      let new_page_h =
        if active_page <= item_sizes.len() { item_sizes[active_page - 1].height } else { px(0.0) };

      let target_y = new_page_y + new_page_h * fraction;
      self.scroll_handle.set_offset(point(px(0.0), -target_y));
    } else if page_changed_programmatically {
      self.last_rendered_page = active_page;
      let target_y = page_y_offset(active_page);
      self.scroll_handle.set_offset(point(px(0.0), -target_y));
    }

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
        view.visible_start.store(visible_range.start, Ordering::Relaxed);
        view.visible_end.store(visible_range.end, Ordering::Relaxed);

        // Sync top visible page with state.current_page during manual scroll,
        // but ignore intermediate shifts caused by zoom transitions.
        let top_visible_page = visible_range.start + 1;
        let should_sync = !zoom_changed
          && (top_visible_page != view.last_rendered_page)
          && (top_visible_page <= total_pages)
          && (top_visible_page > 0);
        if should_sync {
          view.last_rendered_page = top_visible_page;
          state_entity.update(cx, |s, cx| {
            s.current_page = top_visible_page;
            cx.notify();
          });
        }

        let mut page_elements = Vec::new();

        if let Some(book_path) = &book_path_opt {
          let mut cache_lock = cache_arc.lock();

          for page_idx in visible_range {
            let page_num = page_idx + 1;
            if page_num > total_pages {
              break;
            }

            let scale_factor = window.scale_factor();
            let target_dpi =
              crate::ui::pages::book_viewer::loader::compute_target_dpi(zoom_factor, scale_factor);
            let cached_dpi = cache_lock.get_image_dpi(page_num);
            let is_stale_dpi = cached_dpi.is_some_and(|d| d != target_dpi);

            // Image cache lookup / trigger
            let (img, is_img_loading) = match cache_lock.get_image(page_num) {
              Some(PageImageState::Loaded(img)) if !is_stale_dpi => (Some(img.clone()), false),
              Some(PageImageState::Loaded(img)) => {
                let _ = load_tx.send(PageLoadRequest {
                  page: page_num,
                  book_path: book_path.clone(),
                  dpi: target_dpi,
                  book_size,
                  book_hash: book_hash.clone(),
                });
                (Some(img.clone()), false)
              }
              Some(PageImageState::Loading) => (None, true),
              Some(PageImageState::Failed(_)) => (None, false),
              Some(PageImageState::Unloaded) | None => {
                cache_lock.insert_image(page_num, PageImageState::Loading);
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

            // Structured text cache lookup
            let stext = match cache_lock.get_text(page_num) {
              Some(PageTextState::Loaded(t)) => Some(t.clone()),
              _ => None,
            };

            // Links cache lookup
            let links = match cache_lock.get_links(page_num) {
              Some(PageLinksState::Loaded(l)) => Some(l.clone()),
              _ => None,
            };

            let selection_handle = state_entity
              .update(cx, |s, cx| s.get_or_create_selection_handle(page_num, window, cx));

            let page_view = PageView::new(
              page_num,
              state_entity.clone(),
              img,
              is_img_loading,
              stext,
              links,
              selection_handle,
            );

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
        }

        page_elements
      },
    )
    .track_scroll(&scroll)
    .size_full();

    div()
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
  }
}
