use crate::theme::set_app_theme;
use crate::{ctx::GlobalCTX, db::models::AppTheme};
use gpui::*;
use gpui::{App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window};
use gpui_component::{select::*, *};
use rust_i18n::t;

pub struct ThemeSelect {
  theme_select: Entity<SelectState<SearchableVec<AppTheme>>>,
}
impl ThemeSelect {
  pub fn new(window: &mut Window, cx: &mut App) -> Entity<Self> {
    let themes = SearchableVec::new(AppTheme::all());
    let current_theme = cx.ctx().theme();
    let initial_index =
      AppTheme::all().iter().position(|t| t == &current_theme).map(|i| IndexPath::default().row(i));
    let theme_select =
      cx.new(|cx| SelectState::new(themes, initial_index, window, cx).searchable(true));

    cx.new(|cx| {
      fn fun_name(
        _this: &mut ThemeSelect, _state: &Entity<SelectState<SearchableVec<AppTheme>>>,
        event: &SelectEvent<SearchableVec<AppTheme>>, _window: &mut Window,
        cx: &mut Context<'_, ThemeSelect>,
      ) {
        let SelectEvent::Confirm(new_theme) = event;
        if let Some(new_theme) = new_theme {
          cx.ctx_mut().settings.set_theme(new_theme).unwrap();
          set_app_theme(cx, new_theme.to_string());
        };
      }
      cx.subscribe_in(&theme_select, window, fun_name).detach();

      Self { theme_select }
    })
  }
}

impl Render for ThemeSelect {
  fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
    div().child(
      Select::new(&self.theme_select)
        .placeholder(t!("components.theme_select.placeholder"))
        .search_placeholder(t!("components.theme_select.search_placeholder"))
        .w(px(210.0)),
    )
  }
}
