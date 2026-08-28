use crate::ui::pages::book_viewer::constants::{RADIUS_SM, bookmarks};
use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::*;
use gpui_component::ActiveTheme;
use gpui_component::input::{Input, InputEvent, InputState};
use rust_i18n::t;

pub struct BookmarkSearchInput {
  state: Entity<BookViewerState>,
  input_state: Entity<InputState>,
  _subscription: Subscription,
}

impl BookmarkSearchInput {
  pub fn new(window: &mut Window, cx: &mut App, state: Entity<BookViewerState>) -> Entity<Self> {
    let input_state = cx.new(|cx| {
      InputState::new(window, cx)
        .placeholder(t!("components.book_viewer.bookmarks.search_placeholder"))
    });

    let state_clone = state.clone();
    cx.new(|cx: &mut Context<Self>| {
      let subscription = cx.subscribe_in(&input_state, window, {
        let state = state_clone.clone();
        move |_this: &mut Self, input: &Entity<InputState>, ev: &InputEvent, _window, cx| {
          if let InputEvent::Change = ev {
            let query = input.read(cx).text().to_string();
            state.update(cx, |s, cx| {
              s.set_bookmark_search_query(query);
              cx.notify();
            });
          }
        }
      });

      Self { state, input_state, _subscription: subscription }
    })
  }
}

impl Render for BookmarkSearchInput {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let theme = cx.theme();

    div()
      .w_full()
      .h(bookmarks::SEARCH_INPUT_HEIGHT)
      .border_1()
      .border_color(theme.border)
      .hover(move |s| s.border_color(theme.primary))
      .rounded(RADIUS_SM)
      .bg(theme.background)
      .flex()
      .items_center()
      .child(
        Input::new(&self.input_state)
          .appearance(false)
          .text_sm()
          .text_color(theme.foreground)
          .w_full(),
      )
  }
}
