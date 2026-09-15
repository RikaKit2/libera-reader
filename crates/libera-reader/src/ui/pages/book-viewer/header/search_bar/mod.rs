pub mod search_close_btn;
pub mod search_counter;
pub mod search_input;
pub mod search_next_btn;
pub mod search_prev_btn;

pub use search_close_btn::SearchCloseBtn;
pub use search_counter::SearchCounter;
pub use search_input::SearchInput;
pub use search_next_btn::SearchNextBtn;
pub use search_prev_btn::SearchPrevBtn;

use crate::ui::pages::book_viewer::constants::search_bar;
use crate::ui::pages::book_viewer::state::{BookViewerState, SearchHit};
use gpui::*;
use gpui_component::ActiveTheme;
pub struct SearchBar {
  state: Entity<BookViewerState>,
  input: Entity<SearchInput>,
  prev_btn: Entity<SearchPrevBtn>,
  next_btn: Entity<SearchNextBtn>,
  counter: Entity<SearchCounter>,
  close_btn: Entity<SearchCloseBtn>,
}

impl SearchBar {
  pub fn new(window: &mut Window, cx: &mut App, state: Entity<BookViewerState>) -> Entity<Self> {
    let input = SearchInput::new(window, cx, state.clone());
    let prev_btn = cx.new(|_cx| SearchPrevBtn::new(state.clone()));
    let next_btn = cx.new(|_cx| SearchNextBtn::new(state.clone()));
    let counter = cx.new(|_cx| SearchCounter::new(state.clone()));
    let close_btn = cx.new(|_cx| SearchCloseBtn::new(state.clone()));

    cx.new(|_cx| Self { state, input, prev_btn, next_btn, counter, close_btn })
  }
}

impl Render for SearchBar {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let theme = cx.theme();
    let is_open = self.state.read(cx).search_open;

    if !is_open {
      return div();
    }

    let has_query = !self.state.read(cx).search_query.is_empty();

    let mut bar = div()
      .w_full()
      .h(search_bar::HEIGHT)
      .bg(theme.title_bar)
      .border_b_1()
      .border_color(theme.border)
      .flex()
      .items_center()
      .px(search_bar::PADDING_X)
      .gap_x(search_bar::GAP_X)
      .child(self.input.clone());

    if has_query {
      bar = bar.child(self.counter.clone());
    }

    bar.child(self.next_btn.clone()).child(self.prev_btn.clone()).child(self.close_btn.clone())
  }
}

pub fn execute_search(state: Entity<BookViewerState>, cx: &mut App) {
  let (book_path_opt, query, needs_new_search) = state.update(cx, |s, cx| {
    let q = s.search_query.trim().to_string();
    if q.is_empty() {
      s.clear_search();
      cx.notify();
      return (None, String::new(), false);
    }

    if s.search_results.is_empty() {
      s.is_searching = true;
      cx.notify();
      (s.current_book.as_ref().map(|b| b.as_pathbuf()), q, true)
    } else {
      s.next_search_match();
      cx.notify();
      (None, q, false)
    }
  });

  if needs_new_search && let Some(book_path) = book_path_opt {
    let state_worker = state.clone();
    cx.spawn(|async_app: &mut AsyncApp| {
      let mut owned_app = async_app.clone();
      async move {
        let query_clone = query.clone();
        let matches_res = owned_app
          .background_executor()
          .spawn(async move { mutool::search_document_text(&book_path, &query_clone) })
          .await;

        let hits: Vec<SearchHit> = match matches_res {
          Ok(matches) => matches
            .into_iter()
            .map(|m| SearchHit { page: m.page, text: m.snippet.into(), bbox: Some(m.bbox) })
            .collect(),
          Err(_) => Vec::new(),
        };

        state_worker.update(&mut owned_app, |s, cx| {
          s.set_search_results(hits);
          cx.notify();
        });
      }
    })
    .detach();
  }
}
