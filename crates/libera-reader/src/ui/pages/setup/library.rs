use gpui::prelude::FluentBuilder;
use gpui::{
  AbsoluteLength, App, AppContext, Context, Entity, IntoElement, ParentElement, Pixels, Render, Styled, Window,
  div, px,
};
use gpui_component::ActiveTheme;
use gpui_component::{
  Disableable, Icon, IconName, Sizable,
  button::{Button, ButtonVariants},
};
use libera_reader_core::ctx::GlobalCTX;
use rfd::AsyncFileDialog;
use rust_i18n::t;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::ui::pages::setup::{back_btn, next_btn};

static DIALOG_OPEN: AtomicBool = AtomicBool::new(false);

pub(crate) struct Library {}

impl Library {
  pub(crate) fn new(cx: &mut App) -> Entity<Self> {
    cx.new(|_| Self {})
  }
}

impl Render for Library {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let dialog_open = DIALOG_OPEN.load(Ordering::Relaxed);
    let theme = cx.theme();

    let page = div().w_full().h_full().flex().flex_col().children([
      div().flex().justify_center().items_center().flex_col().children([
        div().child(Icon::new(IconName::Folder).with_size(px(40.0))),
        div()
          .mt_2()
          .flex()
          .justify_center()
          .items_center()
          .child(t!("pages.setup.pages.library.title").to_string()),
        div()
          .mt_2()
          .child(t!("pages.setup.pages.library.description").to_string())
          .text_size(AbsoluteLength::Pixels(Pixels::from(14.0))),
      ]),
      div().flex_1().children([
        div().flex().items_center().gap_1().children([
          div().child(Icon::new(IconName::FolderOpen).small()),
          div().child(t!("pages.setup.pages.library.target_dir_label").to_string()),
          div().child(
            Button::new("select_folder_btn")
              .primary()
              .label(t!("pages.setup.pages.library.select_btn"))
              .disabled(dialog_open)
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
          ),
        ]),
        div().flex().items_center().gap_1().children([
          div().child(Icon::new(IconName::Folder).small()),
          div().child(t!("pages.setup.pages.library.target_path_status").to_string()),
          div().children([div()
            .when(cx.ctx().settings.read().path_to_scan.is_some(), |_| {
              div().child(cx.ctx().settings.get_path_to_scan_str().unwrap())
            })
            .when(cx.ctx().settings.read().path_to_scan.is_none(), |_| {
              div().child(t!("pages.setup.pages.library.path_not_selected").to_string())
            })]),
        ]),
      ]),
    ]);

    let path_selected = cx.ctx().settings.read().path_to_scan.is_some();
    let next_btn_disabled = !path_selected;

    div()
      .bg(theme.background)
      .w_full()
      .h_full()
      .p_6()
      .flex()
      .flex_col()
      .justify_between()
      .text_color(theme.foreground)
      .children([
        div().mx_1_6().child(page),
        div().w_full().flex().justify_between().children([back_btn(), next_btn(next_btn_disabled)]),
      ])
  }
}
