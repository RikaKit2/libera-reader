use crate::ui::components::cache_size_select::CacheSizeSelect;
use crate::ui::components::columns_select::ColumnsSelect;
use crate::ui::components::{
  lang_select::LangSelect, path_select::PathSelect, theme_select::ThemeSelect,
  workers_select::WorkersSelect,
};

use gpui::{
  App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window, div, px,
};
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
  fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
    div().flex().flex_col().mt_2().pl_1_4().gap_y_2().children([
      div().child(t!("components.lang_select.header").to_string()),
      div().child(self.lang_select.clone()),
      div().child(t!("components.theme_select.header").to_string()),
      div().child(self.theme_select.clone()),
      div().child(self.path_select.clone()),
      div().child(t!("components.columns_select.header").to_string()),
      div().child(self.columns_select.clone()),
      div().child(t!("components.workers_select.header").to_string()),
      div().w(px(150.0)).child(self.workers_select.clone()),
      div().child(t!("components.cache_size_select.header").to_string()),
      div().w(px(150.0)).child(self.cache_size_select.clone()),
    ])
  }
}
