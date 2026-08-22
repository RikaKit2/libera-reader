pub(crate) mod reverse_btn;
pub(crate) mod sort_controls;

pub(crate) use sort_controls::SortControls;

use crate::app_ext::AppExt;
use crate::books_state::TargetList;
use crate::ui::constants as C;
use gpui::*;
use gpui_component::{
  ActiveTheme,
  input::{Input, InputEvent, InputState},
};
use rust_i18n::t;

/// Reusable top bar: search input + sort controls (all inline, monolithic).
/// Usage: `cx.new(|cx| TopBar::new(window, cx, books_state, target))`
pub(crate) struct TopBar {
  input_state: Entity<InputState>,
  sort_controls: Entity<SortControls>,
  _subscriptions: Vec<Subscription>,
}

impl TopBar {
  pub(crate) fn new(window: &mut Window, cx: &mut Context<Self>, target: TargetList) -> Self {
    let books_state = cx.books_state_entity().clone();
    let input_state: Entity<InputState> =
      cx.new(|cx| InputState::new(window, cx).placeholder(t!("components.search_placeholder")));
    let sort_controls = SortControls::new(window, cx, target);

    let _subscriptions = vec![cx.subscribe_in(&input_state, window, {
      let books_state = books_state.clone();
      move |_this, input_state: &Entity<InputState>, ev: &InputEvent, _window, cx| {
        if let InputEvent::Change = ev {
          let query = input_state.read(cx).text().to_string();
          books_state.update(cx, |state, cx| {
            state.set_search_query(query, target, cx);
          });
        }
      }
    })];

    Self { input_state, sort_controls, _subscriptions }
  }
}

impl Render for TopBar {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let bg = cx.theme().background;
    div()
      .bg(cx.theme().border)
      .w_full()
      .h_12()
      .flex()
      .items_center()
      .gap(px(C::TOP_BAR_GAP))
      .children([
        div().flex_1().child(Input::new(&self.input_state).w_full().bg(bg)),
        div().child(self.sort_controls.clone()),
      ])
  }
}
