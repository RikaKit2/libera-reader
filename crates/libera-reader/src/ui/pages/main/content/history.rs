use gpui::{AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Subscription, Window, div};
use gpui_component::{
  ActiveTheme,
  input::{Input, InputEvent, InputState},
};
use rust_i18n::t;

pub(crate) struct History {
  input_state: Entity<InputState>,
  _subscriptions: Vec<Subscription>,
}
impl History {
  pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
    let input_state = cx.new(|cx| InputState::new(window, cx).placeholder(t!("components.search_placeholder")));

    let _subscriptions = vec![cx.subscribe_in(&input_state, window, {
      // let input_state = input_state.clone();
      move |_this, _, ev: &InputEvent, _window, cx| {
        if let InputEvent::Change = ev {
          // let _value = input_state.read(cx).value();
          cx.notify()
        }
      }
    })];

    Self { input_state, _subscriptions }
  }
}

impl Render for History {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    div().w_full().h_full().flex().flex_col().text_color(cx.theme().foreground).children([
      div().bg(cx.theme().border).w_full().h_12().flex().items_center().child(Input::new(&self.input_state)),
      div().bg(cx.theme().background).w_full().h_full(),
    ])
  }
}
