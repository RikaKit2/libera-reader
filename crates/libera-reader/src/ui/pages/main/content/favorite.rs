use gpui::{
  AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Subscription, Window, div, px,
};
use gpui_component::{
  ActiveTheme, VirtualListScrollHandle,
  input::{Input, InputEvent, InputState},
};
use libera_reader_core::ctx::Ctx;
use rust_i18n::t;

use crate::books_state::{BooksState, TargetList};
use crate::ui::components::{BookGridConfig, SortControls, book_virtual_grid};
use crate::ui::constants as C;

pub(crate) struct Favorite {
  input_state: Entity<InputState>,
  sort_controls: Entity<SortControls>,
  books_state: Entity<BooksState>,
  scroll_handle: VirtualListScrollHandle,
  _subscriptions: Vec<Subscription>,
}

impl Favorite {
  pub fn new(window: &mut Window, cx: &mut Context<Self>, books_state: Entity<BooksState>) -> Self {
    #[rustfmt::skip]
    let input_state: Entity<InputState> = cx.new(|cx|
        InputState::new(window, cx).placeholder(t!("components.search_placeholder")));

    let sort_controls = SortControls::new(window, cx, books_state.clone(), TargetList::Favorites);

    let _subscriptions = vec![cx.subscribe_in(&input_state, window, {
      move |_this, _, ev: &InputEvent, _window, cx| {
        if let InputEvent::Change = ev {
          cx.notify()
        }
      }
    })];

    Self { input_state, sort_controls, books_state, scroll_handle: VirtualListScrollHandle::new(), _subscriptions }
  }
}

impl Render for Favorite {
  fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let (columns, ui_zoom) = {
      let settings = Ctx::global(cx).settings.read();
      (settings.number_of_columns as usize, settings.ui_zoom)
    };

    let available_width =
      window.viewport_size().width - px(C::SIDEBAR_W + C::GRID_PL + C::GRID_PR + C::SCROLLBAR_W);
    let col_width = available_width / columns as f32;
    let row_height = col_width * C::COVER_RATIO * ui_zoom as f32 + px(C::GRID_ROW_HEIGHT_EXTRA);

    div().w_full().h_full().flex().flex_col().text_color(cx.theme().foreground).children([
      div()
        .bg(cx.theme().border)
        .w_full()
        .h_12()
        .flex()
        .items_center()
        .gap(px(C::TOP_BAR_GAP))
        .children([div().flex_1().child(Input::new(&self.input_state)), div().child(self.sort_controls.clone())]),
      div().bg(cx.theme().background).w_full().h_full().pt(px(C::GRID_PT)).child(book_virtual_grid(
        BookGridConfig {
          view: &cx.entity(),
          id: "favorite-virtual-grid".into(),
          state: &self.books_state,
          target: TargetList::Favorites,
          columns,
          row_height,
          scroll_handle: &self.scroll_handle,
        },
        cx,
      )),
    ])
  }
}
