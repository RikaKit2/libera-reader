use crate::ui::components::cache_size_select::CacheSizeSelect;
use crate::ui::components::columns_select::ColumnsSelect;
use crate::ui::components::{
  lang_select::LangSelect, path_select::PathSelect, theme_select::ThemeSelect,
  workers_select::WorkersSelect,
};

use gpui::{
  App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window, div,
};
use gpui_component::{ActiveTheme, StyledExt};
use rust_i18n::t;

pub(crate) struct Settings {
  theme_select: Entity<ThemeSelect>,
  lang_select: Entity<LangSelect>,
  columns_select: Entity<ColumnsSelect>,
  path_select: Entity<PathSelect>,
  workers_select: Entity<WorkersSelect>,
  cache_size_select: Entity<CacheSizeSelect>,
}

impl Settings {
  pub(crate) fn new(window: &mut Window, cx: &mut App) -> Entity<Self> {
    cx.new(|c| Self {
      theme_select: ThemeSelect::new(window, c),
      lang_select: LangSelect::new(window, c),
      columns_select: ColumnsSelect::new(window, c),
      path_select: PathSelect::new(c),
      workers_select: WorkersSelect::new(window, c),
      cache_size_select: CacheSizeSelect::new(window, c),
    })
  }
}

impl Render for Settings {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let theme = cx.theme();

    let settings_list = div().flex().flex_col().gap_3().children([
      div().flex().flex_col().gap_1().children([
        div().text_sm().font_medium().child(t!("components.lang_select.header").to_string()),
        div().child(self.lang_select.clone()),
      ]),
      div().flex().flex_col().gap_1().children([
        div().text_sm().font_medium().child(t!("components.theme_select.header").to_string()),
        div().child(self.theme_select.clone()),
      ]),
      div().flex().flex_col().gap_1().children([
        div().text_sm().font_medium().child(t!("components.path_select.target_path").to_string()),
        div().child(self.path_select.clone()),
      ]),
      div().flex().flex_col().gap_1().children([
        div().text_sm().font_medium().child(t!("components.columns_select.header").to_string()),
        div().child(self.columns_select.clone()),
      ]),
      div().flex().flex_col().gap_1().children([
        div().text_sm().font_medium().child(t!("components.workers_select.header").to_string()),
        div().child(self.workers_select.clone()),
      ]),
      div().flex().flex_col().gap_1().children([
        div().text_sm().font_medium().child(t!("components.cache_size_select.header").to_string()),
        div().child(self.cache_size_select.clone()),
      ]),
    ]);

    div()
      .size_full()
      .bg(theme.background)
      .text_color(theme.foreground)
      .flex()
      .flex_col()
      .items_center()
      .justify_center()
      .child(settings_list)
  }
}
