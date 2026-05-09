use crate::books_state::{BooksState, TargetList};
use crate::ui::components::{BooksGrid, SortControls};
use crate::ui::constants as C;
use gpui::{
  AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Subscription, Window,
  div, px,
};
use gpui_component::{
  ActiveTheme,
  input::{Input, InputEvent, InputState},
};
use libera_reader_core::ctx::Ctx;
use rust_i18n::t;

pub(crate) struct Library {
  input_state: Entity<InputState>,
  sort_controls: Entity<SortControls>,
  book_grid: Entity<BooksGrid>,
  _subscriptions: Vec<Subscription>,
}

impl Library {
  pub fn new(window: &mut Window, cx: &mut Context<Self>, books_state: Entity<BooksState>) -> Self {
    let input_state: Entity<InputState> =
      cx.new(|cx| InputState::new(window, cx).placeholder(t!("components.search_placeholder")));
    let sort_controls = SortControls::new(window, cx, books_state.clone(), TargetList::Library);

    let book_grid = cx.new(|cx| {
      BooksGrid::new(
        books_state.clone(),
        TargetList::Library,
        1,
        px(0.0),
        "library-virtual-grid".into(),
        cx,
      )
    });

    let _subscriptions = vec![cx.subscribe_in(&input_state, window, {
      let books_state = books_state.clone();
      move |_this, input_state: &Entity<InputState>, ev: &InputEvent, _window, cx| {
        if let InputEvent::Change = ev {
          let query = input_state.read(cx).text().to_string();
          books_state.update(cx, |state, cx| {
            state.set_search_query(query, TargetList::Library, cx);
          });
        }
      }
    })];
    Self { input_state, sort_controls, book_grid, _subscriptions }
  }
}

impl Render for Library {
  fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let (columns, ui_zoom) = {
      let settings = Ctx::global(cx).settings.read();
      (settings.number_of_columns as usize, settings.ui_zoom)
    };
    let available_width =
      window.viewport_size().width - px(C::SIDEBAR_W + C::GRID_PL + C::GRID_PR + C::SCROLLBAR_W);
    let col_width = available_width / columns as f32;
    let row_height = col_width * C::COVER_RATIO * ui_zoom as f32 + px(C::GRID_ROW_HEIGHT_EXTRA);
    let mode = Ctx::global(cx).settings.read().card_display_mode;
    self.book_grid.update(cx, |grid, _cx| {
      grid.set_layout(columns, row_height, mode);
    });
    div().w_full().h_full().flex().flex_col().text_color(cx.theme().foreground).children([
      div()
        .bg(cx.theme().border)
        .w_full()
        .h_12()
        .flex()
        .items_center()
        .gap(px(C::TOP_BAR_GAP))
        .children([
          div().flex_1().child(Input::new(&self.input_state)),
          div().child(self.sort_controls.clone()),
        ]),
      div()
        .bg(cx.theme().background)
        .w_full()
        .h_full()
        .pt(px(C::GRID_PT))
        .child(self.book_grid.clone()),
    ])
  }
}
