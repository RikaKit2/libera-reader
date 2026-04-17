use gpui::prelude::FluentBuilder;
use gpui::{App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window, div, px};
use gpui_component::{
  Disableable,
  button::{Button, ButtonVariants},
};
use libera_reader_core::ctx::GlobalCTX;
use rfd::AsyncFileDialog;
use rust_i18n::t;
use std::sync::atomic::{AtomicBool, Ordering};

static DIALOG_OPEN: AtomicBool = AtomicBool::new(false);

pub struct PathSelect {}

impl PathSelect {
  pub fn new(cx: &mut App) -> Entity<Self> {
    cx.new(|_| Self {})
  }
}

impl Render for PathSelect {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let dialog_open = DIALOG_OPEN.load(Ordering::Relaxed);

    let header = div().child(t!("components.path_select.target_path").to_string());

    let btn = div().child(
      Button::new("select_folder_btn")
        .primary()
        .label(t!("components.path_select.select_btn"))
        .disabled(dialog_open)
        .w(px(200.0))
        .on_click(|_event, _window, cx| {
          if DIALOG_OPEN.load(Ordering::Relaxed) {
            return; // Dialog already open, don't open another
          }
          DIALOG_OPEN.store(true, Ordering::Relaxed);
          cx.spawn(async move |cx| {
            if let Some(folder) = AsyncFileDialog::new().pick_folder().await {
              let path = folder.path().to_path_buf();
              let _ = cx.update(|cx| {
                cx.ctx_mut().settings.set_path_to_scan(path).unwrap();
              });
            }
            // Reset dialog state when done
            DIALOG_OPEN.store(false, Ordering::Relaxed);
          })
          .detach();
        }),
    );

    let path_to_scan = div()
      .when(cx.ctx().settings.read().path_to_scan.is_some(), |_| {
        div().child(cx.ctx().settings.get_path_to_scan_str().unwrap())
      })
      .when(cx.ctx().settings.read().path_to_scan.is_none(), |_| {
        div().child(t!("components.path_select.path_not_selected").to_string())
      })
      .max_w(px(300.0));

    div().flex().flex_col().gap_2().children([div().flex().gap_x_2().children([header, path_to_scan]), btn])
  }
}
