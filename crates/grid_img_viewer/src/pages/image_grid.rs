use crate::TOKIO;
use crate::cache::{BoundedCache, CoverState};
use crate::image_utils::{ROW_H, is_image};
use crate::loader::spawn_background_loader;
use gpui::prelude::*;
use gpui::{
  AsyncApp, Context, Div, ImageSource, IntoElement, ObjectFit, ParentElement, Pixels, Render,
  SharedString, Styled, Window, div, px, size,
};
use gpui_component::scroll::Scrollbar;
use gpui_component::{ActiveTheme, VirtualListScrollHandle, v_virtual_list};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

const COLS: usize = 6;

pub struct ImageGridPage {
  images: Vec<PathBuf>,
  item_sizes: Rc<Vec<gpui::Size<Pixels>>>,
  scroll_handle: VirtualListScrollHandle,
  image_cache: Arc<Mutex<BoundedCache>>,
  visible_start: Arc<AtomicUsize>,
  visible_end: Arc<AtomicUsize>,
  load_tx: tokio::sync::mpsc::UnboundedSender<(usize, PathBuf)>,
}

impl ImageGridPage {
  pub fn new(folder: &PathBuf, cache_size: usize, cx: &mut Context<Self>) -> Self {
    let mut images = Vec::new();
    if let Ok(entries) = std::fs::read_dir(folder) {
      for entry in entries.flatten() {
        let p = entry.path();
        if is_image(&p) {
          images.push(p);
        }
      }
    }
    images.sort();

    let rows = images.len().div_ceil(COLS);
    let sizes = Rc::new(std::iter::repeat_n(size(px(800.0), px(ROW_H)), rows.max(1)).collect());

    let cache = Arc::new(Mutex::new(BoundedCache::new(cache_size.max(1))));
    let visible_start = Arc::new(AtomicUsize::new(0));
    let visible_end = Arc::new(AtomicUsize::new(0));

    let (load_tx, load_rx) = tokio::sync::mpsc::unbounded_channel::<(usize, PathBuf)>();
    let (notify_tx, mut notify_rx) = tokio::sync::mpsc::unbounded_channel::<()>();

    cx.spawn(|this: gpui::WeakEntity<Self>, cx: &mut AsyncApp| {
      let mut owned_cx = cx.clone();
      async move {
        while notify_rx.recv().await.is_some() {
          let _ = this.update(&mut owned_cx, |_, cx| cx.notify());
        }
      }
    })
    .detach();

    let tokio_runtime = TOKIO.get().expect("Tokio runtime not initialized");
    spawn_background_loader(
      tokio_runtime,
      cache.clone(),
      visible_start.clone(),
      visible_end.clone(),
      load_rx,
      notify_tx,
    );

    Self {
      images,
      item_sizes: sizes,
      scroll_handle: VirtualListScrollHandle::new(),
      image_cache: cache,
      visible_start,
      visible_end,
      load_tx,
    }
  }

  fn render_cell(
    &self, idx: usize, state: Option<CoverState>, bg_color: gpui::Hsla, muted: gpui::Hsla,
  ) -> Div {
    let path = &self.images[idx];
    let name: SharedString =
      path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string().into();

    let content: Div = match state {
      // ⚡️ MIGRATION: Draw texture directly via ImageSource::Render
      Some(CoverState::Loaded(loaded_img)) => div().w_full().h_full().child(
        gpui::img(ImageSource::Render(loaded_img)).w_full().h_full().object_fit(ObjectFit::Cover),
      ),
      Some(CoverState::Failed) => div()
        .w_full()
        .h_full()
        .flex()
        .items_center()
        .justify_center()
        .text_color(gpui::red())
        .child(SharedString::from("Error")),
      Some(CoverState::Loading) | None => div()
        .w_full()
        .h_full()
        .flex()
        .items_center()
        .justify_center()
        .text_color(muted)
        .child(SharedString::from("...")),
    };

    div()
      .flex()
      .flex_col()
      .flex_1()
      .h(px(150.0))
      .gap_1()
      .child(div().w_full().flex_1().rounded_md().overflow_hidden().bg(bg_color).child(content))
      .child(div().text_xs().text_color(muted).truncate().child(name))
  }
}

impl Render for ImageGridPage {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    // On each frame render, flush accumulated evicted images from VRAM
    self.image_cache.lock().unwrap().flush_evictions(cx);

    let total = self.images.len();
    let total_rows = total.div_ceil(COLS);
    let scroll = self.scroll_handle.clone();
    let bg_color = cx.theme().background;
    let muted = cx.theme().muted_foreground;

    let list = v_virtual_list(
      cx.entity().clone(),
      "img-grid",
      self.item_sizes.clone(),
      move |view: &mut ImageGridPage, visible_range, _window, _cx| {
        let start_idx = visible_range.start * COLS;
        let end_idx = (visible_range.end * COLS).min(total);
        view.visible_start.store(start_idx, Ordering::Relaxed);
        view.visible_end.store(end_idx, Ordering::Relaxed);

        let mut cache_lock = view.image_cache.lock().unwrap();
        let mut rows = Vec::new();

        for row in visible_range {
          if row >= total_rows {
            break;
          }
          let start = row * COLS;
          let end = (start + COLS).min(total);

          let mut cells = Vec::with_capacity(end - start);
          for i in start..end {
            let state = cache_lock.get_mut(i);

            if state.is_none() {
              cache_lock.insert(i, CoverState::Loading);
              let _ = view.load_tx.send((i, view.images[i].clone()));
            }
            cells.push(view.render_cell(i, state, bg_color, muted));
          }
          rows.push(div().flex().gap_2().h_full().w_full().px_2().children(cells));
        }
        rows
      },
    )
    .track_scroll(&scroll)
    .flex_1()
    .w_full();

    div().relative().size_full().child(list).child(
      div()
        .absolute()
        .right_0()
        .top_0()
        .bottom_0()
        .w_2()
        .bg(cx.theme().scrollbar)
        .child(Scrollbar::new(&self.scroll_handle)),
    )
  }
}
