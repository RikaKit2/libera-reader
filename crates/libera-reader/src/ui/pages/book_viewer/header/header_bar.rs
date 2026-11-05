use crate::ui::pages::book_viewer::constants::header;
use crate::ui::pages::book_viewer::header::controls::*;
use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::*;
use gpui_component::ActiveTheme;

pub struct HeaderBar {
  state: Entity<BookViewerState>,
  page_number_input: Entity<PageNumberInput>,
  zoom_select: Entity<ZoomSelect>,
  _subscription: Subscription,
}

impl HeaderBar {
  pub fn new(window: &mut Window, cx: &mut App, state: Entity<BookViewerState>) -> Entity<Self> {
    let page_number_input = PageNumberInput::new(window, cx, state.clone());
    let zoom_select = ZoomSelect::new(window, cx, state.clone());

    cx.new(|cx| {
      let sub = cx.observe(&state, |_, _, cx| cx.notify());
      Self { state, page_number_input, zoom_select, _subscription: sub }
    })
  }
}

impl Render for HeaderBar {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let theme = cx.theme();

    div()
      .w_full()
      .h(header::HEIGHT)
      .bg(theme.title_bar)
      .border_b_1()
      .border_color(theme.border)
      .flex()
      .items_center()
      .justify_between()
      .px(header::PADDING_X)
      // Left section (3 buttons)
      .child(
        div()
          .flex()
          .items_center()
          .gap_x(header::GAP_TIGHT)
          .child(OutlineBtn::new(self.state.clone()))
          .child(ScrollModeBtn::new(self.state.clone()))
          .child(TtsBtn::new(self.state.clone())),
      )
      // Center section
      .child(
        div()
          .flex()
          .items_center()
          .gap_x(header::GAP_NORMAL)
          .child(PrevPageBtn::new(self.state.clone()))
          .child(NextPageBtn::new(self.state.clone()))
          .child(self.page_number_input.clone())
          .child(PageCounter::new(self.state.clone()))
          .child(div().ml_1().child(self.zoom_select.clone())),
      )
      // Right section (6 buttons in strict order)
      .child(
        div()
          .flex()
          .items_center()
          .gap_x(header::GAP_TIGHT)
          .child(InvertColorsBtn::new(self.state.clone()))
          .child(BookmarksBtn::new(self.state.clone()))
          .child(ThumbnailsBtn::new(self.state.clone()))
          .child(SearchToggleBtn::new(self.state.clone()))
          .child(FullscreenBtn::new(self.state.clone()))
          .child(ExitBtn::new()),
      )
  }
}
