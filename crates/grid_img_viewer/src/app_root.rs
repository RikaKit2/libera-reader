use crate::pages::{
  FolderSelectPage, ImageGridPage,
  folder_select::{CacheSize, SelectedFolder},
};
use gpui::prelude::*;
use gpui::{Context, Entity, IntoElement, ParentElement, Render, Styled, Window, div, px};
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::{ActiveTheme, Disableable};

enum Page {
  Folder(Entity<FolderSelectPage>),
  Grid(Entity<ImageGridPage>),
}

pub struct AppRoot {
  page: Page,
}

impl AppRoot {
  pub fn new(cx: &mut Context<Self>) -> Self {
    cx.set_global(SelectedFolder(std::path::PathBuf::new()));
    Self { page: Page::Folder(cx.new(|_| FolderSelectPage::new())) }
  }
}

impl Render for AppRoot {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    match &self.page {
      Page::Folder(folder_entity) => {
        let has_path =
          cx.try_global::<SelectedFolder>().map(|g| !g.0.as_os_str().is_empty()).unwrap_or(false);
        let next_btn = div().child(
          Button::new("go-next")
            .primary()
            .label("Next →")
            .disabled(!has_path)
            .w(px(120.0))
            .on_click(cx.listener(move |this, _ev, _win, cx| {
              let path = cx.try_global::<SelectedFolder>().map(|g| g.0.clone()).unwrap_or_default();
              if !path.as_os_str().is_empty() {
                let cache_size = cx.try_global::<CacheSize>().map(|c| c.0).unwrap_or(10000);
                this.page = Page::Grid(cx.new(|cx| ImageGridPage::new(&path, cache_size, cx)));
                cx.notify();
              }
            })),
        );
        div().size_full().bg(cx.theme().background).child(
          div()
            .flex()
            .flex_col()
            .size_full()
            .child(
              div()
                .flex()
                .items_center()
                .justify_end()
                .px_4()
                .py_2()
                .border_b_1()
                .border_color(cx.theme().border)
                .child(next_btn),
            )
            .child(div().flex_1().child(folder_entity.clone())),
        )
      }
      Page::Grid(grid_entity) => div().size_full().bg(cx.theme().background).child(
        div()
          .flex()
          .flex_col()
          .size_full()
          .child(
            div()
              .flex()
              .items_center()
              .px_4()
              .py_2()
              .border_b_1()
              .border_color(cx.theme().border)
              .child(Button::new("go-back").ghost().label("← Back").on_click(cx.listener(
                move |this, _ev, _win, cx| {
                  this.page = Page::Folder(cx.new(|_| FolderSelectPage::new()));
                  cx.notify();
                },
              ))),
          )
          .child(div().flex_1().child(grid_entity.clone())),
      ),
    }
  }
}
