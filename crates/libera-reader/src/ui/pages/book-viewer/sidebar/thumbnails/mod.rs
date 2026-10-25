pub mod thumbnail_card;
pub use thumbnail_card::ThumbnailCard;

use crate::TOKIO;
use crate::app_dirs::AppDirs;
use crate::app_ext::AppExt;
use crate::db::models::books::book::BookPath;
use crate::types::HashSet;
use crate::ui::pages::book_viewer::state::BookViewerState;
use crate::utils::image::{PAGE_THUMB_MAX_PX, load_thumbnail_sized};
use gpui::*;
use gpui_component::scroll::Scrollbar;
use gpui_component::{VirtualListScrollHandle, v_virtual_list};
use lru::LruCache;
use parking_lot::Mutex;
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

struct ThumbRequest {
  page: usize,
  book_path: PathBuf,
  book_size: u64,
  book_hash: Option<String>,
  app_dirs: AppDirs,
}

struct SharedThumbCache {
  lru: LruCache<usize, Arc<RenderImage>>,
  in_flight: HashSet<usize>,
  evicted: Vec<Arc<RenderImage>>,
}

impl SharedThumbCache {
  fn new(cap: usize) -> Self {
    Self {
      lru: LruCache::new(NonZeroUsize::new(cap).unwrap()),
      in_flight: HashSet::default(),
      evicted: Vec::new(),
    }
  }

  fn clear(&mut self) {
    for (_, img) in self.lru.iter() {
      self.evicted.push(img.clone());
    }
    self.lru.clear();
    self.in_flight.clear();
  }
}

pub struct ThumbnailsView {
  state: Entity<BookViewerState>,
  scroll_handle: VirtualListScrollHandle,
  cache: Arc<Mutex<SharedThumbCache>>,
  visible_start: Arc<AtomicUsize>,
  visible_end: Arc<AtomicUsize>,
  load_tx: tokio::sync::mpsc::UnboundedSender<ThumbRequest>,
  last_book_path: Option<BookPath>,
  last_current_page: usize,
  cached_item_sizes: Rc<Vec<Size<Pixels>>>,
  cached_total_rows: usize,
}

impl ThumbnailsView {
  pub fn new(state: Entity<BookViewerState>, cx: &mut Context<Self>) -> Self {
    let cache = Arc::new(Mutex::new(SharedThumbCache::new(64)));
    let visible_start = Arc::new(AtomicUsize::new(1));
    let visible_end = Arc::new(AtomicUsize::new(10));
    let (load_tx, mut load_rx) = tokio::sync::mpsc::unbounded_channel::<ThumbRequest>();
    let (notify_tx, mut notify_rx) = tokio::sync::mpsc::unbounded_channel::<()>();

    cx.spawn(|this: WeakEntity<Self>, cx: &mut AsyncApp| {
      let mut owned_cx = cx.clone();
      async move {
        while notify_rx.recv().await.is_some() {
          while notify_rx.try_recv().is_ok() {}
          let _ = this.update(&mut owned_cx, |_, cx| cx.notify());
        }
      }
    })
    .detach();

    let cache_bg = cache.clone();
    let vs = visible_start.clone();
    let ve = visible_end.clone();
    let semaphore = Arc::new(tokio::sync::Semaphore::new(2));

    if let Some(rt) = TOKIO.get() {
      rt.spawn(async move {
        while let Some(req) = load_rx.recv().await {
          let Ok(permit) = semaphore.clone().acquire_owned().await else { break };
          let s = vs.load(Ordering::Relaxed);
          let e = ve.load(Ordering::Relaxed);

          // Мгновенно отменяем миниатюры, которые уже проскроллили мимо!
          if req.page < s.saturating_sub(4) || req.page > e + 4 {
            cache_bg.lock().in_flight.swap_remove(&req.page);
            drop(permit);
            continue;
          }

          let cache_worker = cache_bg.clone();
          let notify_worker = notify_tx.clone();
          tokio::task::spawn_blocking(move || {
            let loaded_img = Self::load_or_render_thumb(&req);
            {
              let mut lock = cache_worker.lock();
              lock.in_flight.swap_remove(&req.page);
              if let Some(img) = loaded_img
                && let Some((_, evicted)) = lock.lru.push(req.page, img)
              {
                lock.evicted.push(evicted);
              }
            }
            let _ = notify_worker.send(());
            drop(permit);
          });
        }
      });
    }

    Self {
      state,
      scroll_handle: VirtualListScrollHandle::new(),
      cache,
      visible_start,
      visible_end,
      load_tx,
      last_book_path: None,
      last_current_page: 0,
      cached_item_sizes: Rc::new(Vec::new()),
      cached_total_rows: 0,
    }
  }

  fn load_or_render_thumb(req: &ThumbRequest) -> Option<Arc<RenderImage>> {
    if req.book_size == 0 {
      return None;
    }
    // Проверяем все бакеты DPI перед запуском mutool
    for &dpi in &[72, 96, 108, 120, 144, 180, 216] {
      let path =
        req.app_dirs.book_page_path(req.book_size, req.book_hash.as_deref(), req.page, dpi);
      if path.exists()
        && let Some(img) = load_thumbnail_sized(&path, PAGE_THUMB_MAX_PX)
      {
        return Some(img);
      }
    }

    let p72 = req.app_dirs.book_page_path(req.book_size, req.book_hash.as_deref(), req.page, 72);
    if mutool::render_page_to_webp(&req.book_path, req.page, 72, &p72, 80.0).is_ok() {
      return load_thumbnail_sized(&p72, PAGE_THUMB_MAX_PX);
    }
    None
  }

  fn flush_evictions(&mut self, cx: &mut App) {
    let evicted = std::mem::take(&mut self.cache.lock().evicted);
    for img in evicted {
      cx.drop_image(img, None);
    }
  }
}

impl Render for ThumbnailsView {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    self.flush_evictions(cx);
    let (total_pages, current_page, current_book, book_size, book_hash) = {
      let state = self.state.read(cx);
      (
        state.total_pages,
        state.current_page,
        state.current_book.clone(),
        state.book_size,
        state.book_hash.clone(),
      )
    };

    if current_book != self.last_book_path {
      self.last_book_path = current_book.clone();
      self.cache.lock().clear();
      self.flush_evictions(cx);
      self.last_current_page = 0;
    }

    let columns = 2;
    let total_rows = total_pages.div_ceil(columns).max(1);
    let row_height = px(172.0);

    if self.cached_total_rows != total_rows {
      self.cached_total_rows = total_rows;
      self.cached_item_sizes =
        Rc::new((0..total_rows).map(|_| size(px(220.), row_height)).collect());
    }
    let item_sizes = self.cached_item_sizes.clone();

    if current_page != self.last_current_page && current_page > 0 {
      self.last_current_page = current_page;
      let target_row = (current_page - 1) / columns;
      self.scroll_handle.scroll_to_item(target_row, ScrollStrategy::Center);
    }

    let state = self.state.clone();
    let scroll = self.scroll_handle.clone();
    let app_dirs = cx.app_dirs().clone();

    let list = v_virtual_list(
      cx.entity().clone(),
      "thumbnails-virtual-list",
      item_sizes,
      move |view: &mut ThumbnailsView, visible_range, _window, cx| {
        let start_p = visible_range.start * columns + 1;
        let end_p = (visible_range.end * columns).min(total_pages).max(start_p);
        view.visible_start.store(start_p, Ordering::Relaxed);
        view.visible_end.store(end_p, Ordering::Relaxed);

        let mut rows = Vec::with_capacity(visible_range.len());
        let mut cache_lock = view.cache.lock();

        for row in visible_range {
          if row >= total_rows {
            break;
          }
          let start_page = row * columns + 1;
          let end_page = ((row + 1) * columns).min(total_pages);
          let mut cards = Vec::with_capacity(end_page.saturating_sub(start_page) + 1);

          for page in start_page..=end_page {
            let is_active = page == current_page;
            let img = if let Some(cached) = cache_lock.lru.get(&page) {
              Some(cached.clone())
            } else {
              if !cache_lock.in_flight.contains(&page)
                && let Some(book) = &current_book
              {
                cache_lock.in_flight.insert(page);
                let _ = view.load_tx.send(ThumbRequest {
                  page,
                  book_path: book.as_pathbuf(),
                  book_size,
                  book_hash: book_hash.clone(),
                  app_dirs: app_dirs.clone(),
                });
              }
              None
            };
            let card = ThumbnailCard::new(page, is_active, img, state.clone());
            cards.push(card.render(cx));
          }
          rows.push(
            div().w_full().h(row_height).flex().flex_row().justify_center().gap_3().children(cards),
          );
        }
        rows
      },
    )
    .track_scroll(&scroll)
    .size_full();

    div().relative().size_full().p_3().child(list).child(
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
