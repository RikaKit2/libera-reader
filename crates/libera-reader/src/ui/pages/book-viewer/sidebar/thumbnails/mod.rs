pub mod thumbnail_card;

pub use thumbnail_card::ThumbnailCard;

use crate::app_ext::AppExt;
use crate::db::models::books::book::BookPath;
use crate::types::HashSet;
use crate::ui::pages::book_viewer::loader::{DiskPageCacheResult, render_and_cache_page};
use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::*;
use gpui_component::{VirtualListScrollHandle, v_virtual_list};
use lru::LruCache;
use std::num::NonZeroUsize;
use std::rc::Rc;
use std::sync::Arc;

pub struct ThumbnailsView {
  state: Entity<BookViewerState>,
  scroll_handle: VirtualListScrollHandle,
  thumbnail_cache: LruCache<usize, Arc<RenderImage>>,
  loading_pages: HashSet<usize>,
  evicted: Vec<Arc<RenderImage>>,
  last_book_path: Option<BookPath>,
  last_current_page: usize,
  thumbnail_semaphore: Arc<tokio::sync::Semaphore>,
}

impl ThumbnailsView {
  pub fn new(state: Entity<BookViewerState>) -> Self {
    Self {
      state,
      scroll_handle: VirtualListScrollHandle::new(),
      thumbnail_cache: LruCache::new(NonZeroUsize::new(32).unwrap()),
      loading_pages: HashSet::default(),
      evicted: Vec::new(),
      last_book_path: None,
      last_current_page: 0,
      thumbnail_semaphore: Arc::new(tokio::sync::Semaphore::new(2)),
    }
  }

  pub fn flush_evictions(&mut self, cx: &mut App) {
    let evicted = std::mem::take(&mut self.evicted);
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

    // If book changed, reset cache and drop evicted images
    if current_book != self.last_book_path {
      self.last_book_path = current_book.clone();
      let evicted = std::mem::take(&mut self.evicted);
      for img in evicted {
        cx.drop_image(img, None);
      }
      for (_, img) in self.thumbnail_cache.iter() {
        cx.drop_image(img.clone(), None);
      }
      self.thumbnail_cache.clear();
      self.loading_pages.clear();
      self.last_current_page = 0;
    }

    let columns = 2;
    let total_rows = total_pages.div_ceil(columns);
    let row_height = px(172.0);
    let item_sizes: Rc<Vec<Size<Pixels>>> =
      Rc::new((0..total_rows.max(1)).map(|_| size(px(0.), row_height)).collect());

    if current_page != self.last_current_page && current_page > 0 {
      self.last_current_page = current_page;
      let target_row = (current_page - 1) / columns;
      self.scroll_handle.scroll_to_item(target_row, ScrollStrategy::Center);
    }

    let state = self.state.clone();
    let scroll = self.scroll_handle.clone();
    let app_dirs = cx.app_dirs().clone();

    div().size_full().p_3().child(
      v_virtual_list(
        cx.entity().clone(),
        "thumbnails-virtual-list",
        item_sizes,
        move |view: &mut ThumbnailsView, visible_range, _window, cx| {
          let mut rows = Vec::with_capacity(visible_range.len());

          for row in visible_range {
            if row >= total_rows {
              break;
            }
            let start_page = row * columns + 1;
            let end_page = ((row + 1) * columns).min(total_pages);

            let mut cards = Vec::with_capacity(end_page - start_page + 1);
            for page in start_page..=end_page {
              let is_active = page == current_page;

              // Lookup or load thumbnail image
              let img = if let Some(cached) = view.thumbnail_cache.get(&page) {
                Some(cached.clone())
              } else {
                if !view.loading_pages.contains(&page)
                  && let Some(book) = &current_book
                {
                  view.loading_pages.insert(page);
                  let page_num = page;
                  let book_path = book.as_pathbuf();
                  let app_dirs_clone = app_dirs.clone();
                  let book_hash_clone = book_hash.clone();
                  let semaphore = view.thumbnail_semaphore.clone();

                  cx.spawn(move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
                    let mut owned_cx = cx.clone();
                    async move {
                      let loaded_img = owned_cx
                        .background_executor()
                        .spawn(async move {
                          if book_size == 0 {
                            return None;
                          }
                          // 1. Try 72 DPI disk cache
                          let p72 = app_dirs_clone.book_page_path(
                            book_size,
                            book_hash_clone.as_deref(),
                            page_num,
                            72,
                          );
                          match DiskPageCacheResult::probe_and_load(&p72) {
                            DiskPageCacheResult::Hit(img) => return Some(img),
                            DiskPageCacheResult::EmptyFile | DiskPageCacheResult::CorruptedFile => {
                              let _ = std::fs::remove_file(&p72);
                            }
                            DiskPageCacheResult::Miss => {}
                          }

                          // 2. Try 150 DPI disk cache
                          let p150 = app_dirs_clone.book_page_path(
                            book_size,
                            book_hash_clone.as_deref(),
                            page_num,
                            150,
                          );
                          match DiskPageCacheResult::probe_and_load(&p150) {
                            DiskPageCacheResult::Hit(img) => return Some(img),
                            DiskPageCacheResult::EmptyFile | DiskPageCacheResult::CorruptedFile => {
                              let _ = std::fs::remove_file(&p150);
                            }
                            DiskPageCacheResult::Miss => {}
                          }
                          // 3. Fallback: generate 72 DPI thumbnail and cache on disk
                          let _permit = semaphore.acquire().await.ok();
                          match render_and_cache_page(&book_path, page_num, 72, &p72) {
                            crate::ui::pages::book_viewer::cache::PageImageState::Loaded(img) => {
                              Some(img)
                            }
                            _ => None,
                          }
                        })
                        .await;

                      let _ = this.update(&mut owned_cx, |this, cx| {
                        this.loading_pages.shift_remove(&page_num);
                        if let Some(img) = loaded_img {
                          if let Some((_, evicted_img)) = this.thumbnail_cache.push(page_num, img) {
                            this.evicted.push(evicted_img);
                          }
                          cx.notify();
                        }
                      });
                    }
                  })
                  .detach();
                }
                None
              };
              let card = ThumbnailCard::new(page, is_active, img, state.clone());
              cards.push(card.render(cx));
            }

            rows.push(
              div()
                .w_full()
                .h(row_height)
                .flex()
                .flex_row()
                .justify_center()
                .gap_3()
                .children(cards),
            );
          }

          rows
        },
      )
      .track_scroll(&scroll)
      .size_full(),
    )
  }
}
