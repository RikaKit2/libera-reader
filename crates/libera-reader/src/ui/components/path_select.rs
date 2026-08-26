use crate::app_ext::AppExt;
use gpui::{
  App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window, div, px,
};
use gpui_component::{
  ActiveTheme, Disableable,
  button::{Button, ButtonVariants},
};
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
    let path_str = cx.settings().get_path_to_scan_str();
    let theme = cx.theme();

    let btn = Button::new("select_folder_btn")
      .primary()
      .label(t!("components.path_select.select_btn"))
      .disabled(dialog_open)
      .w(px(220.0))
      .on_click(|_event, _window, cx| {
        if DIALOG_OPEN.load(Ordering::Relaxed) {
          return;
        }
        DIALOG_OPEN.store(true, Ordering::Relaxed);
        cx.spawn(async move |cx| {
          if let Some(folder) = AsyncFileDialog::new().pick_folder().await {
            let path = folder.path().to_path_buf();
            cx.update(|cx| {
              let _ = cx.services_mut().notify_service.stop();
              cx.settings_mut().set_path_to_scan(path).unwrap();
              crate::services::start_services(cx);
            });
          }
          DIALOG_OPEN.store(false, Ordering::Relaxed);
        })
        .detach();
      });

    let path_display = match path_str {
      Some(path) => div().text_xs().text_color(theme.foreground.opacity(0.85)).child(path),
      None => div()
        .text_xs()
        .text_color(theme.foreground.opacity(0.6))
        .child(t!("components.path_select.path_not_selected").to_string()),
    };

    div().flex().flex_col().items_start().gap_1().children([div().child(btn), path_display])
  }
}
