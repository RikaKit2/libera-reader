use crate::app_ext::AppExt;
use crate::db::models::Lang;
use crate::settings::apply_language;
use gpui::*;
use gpui::{App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window};
use gpui_component::{select::*, *};
use rust_i18n::t;

pub struct LangSelect {
  lang_select: Entity<SelectState<SearchableVec<Lang>>>,
}
impl LangSelect {
  pub fn new(window: &mut Window, cx: &mut App) -> Entity<Self> {
    let langs = SearchableVec::new(Lang::all().to_vec());
    let current_lang = cx.settings().read().language.clone();
    let initial_index =
      Lang::all().iter().position(|t| t == &current_lang).map(|i| IndexPath::default().row(i));
    let lang_select =
      cx.new(|cx| SelectState::new(langs, initial_index, window, cx).searchable(true));

    cx.new(|cx| {
      fn fun_name(
        _this: &mut LangSelect, _state: &Entity<SelectState<SearchableVec<Lang>>>,
        event: &SelectEvent<SearchableVec<Lang>>, _window: &mut Window,
        cx: &mut Context<'_, LangSelect>,
      ) {
        let SelectEvent::Confirm(new_lang) = event;
        if let Some(new_lang) = new_lang {
          cx.settings_mut().set_language(new_lang.clone()).unwrap();
          apply_language(cx);
        };
      }
      cx.subscribe_in(&lang_select, window, fun_name).detach();

      Self { lang_select }
    })
  }
}

impl Render for LangSelect {
  fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
    div().child(
      Select::new(&self.lang_select)
        .placeholder(t!("components.lang_select.placeholder"))
        .search_placeholder(t!("components.lang_select.search_placeholder"))
        .w(px(220.0)),
    )
  }
}
