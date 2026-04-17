use crate::ui::components::columns_select::ColumnsSelect;
use crate::ui::components::{lang_select::LangSelect, path_select::PathSelect, theme_select::ThemeSelect};

use gpui::{App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window, div};
use rust_i18n::t;

pub(crate) struct Settings {
  theme_select: Entity<ThemeSelect>,
  lang_select: Entity<LangSelect>,
  columns_select: Entity<ColumnsSelect>,
  path_select: Entity<PathSelect>,
}

impl Settings {
  pub(crate) fn new(window: &mut Window, cx: &mut App) -> Entity<Self> {
    cx.new(|c| Self {
      theme_select: ThemeSelect::new(window, c),
      lang_select: LangSelect::new(window, c),
      columns_select: ColumnsSelect::new(window, c),
      path_select: PathSelect::new(c),
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
    ])
  }
}
