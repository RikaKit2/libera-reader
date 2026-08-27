use crate::ui::pages::book_viewer::header::controls::*;
use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::*;

pub struct HeaderBar {
  // Left controls
  outline_btn: Entity<OutlineBtn>,
  scroll_mode_btn: Entity<ScrollModeBtn>,
  tts_btn: Entity<TtsBtn>,

  // Center controls
  prev_page_btn: Entity<PrevPageBtn>,
  next_page_btn: Entity<NextPageBtn>,
  page_number_input: Entity<PageNumberInput>,
  page_counter: Entity<PageCounter>,
  zoom_select: Entity<ZoomSelect>,

  // Right controls (in strict Vue order 1..7)
  invert_colors_btn: Entity<InvertColorsBtn>,
  bookmarks_btn: Entity<BookmarksBtn>,
  thumbnails_btn: Entity<ThumbnailsBtn>,
  library_btn: Entity<LibraryBtn>,
  search_toggle_btn: Entity<SearchToggleBtn>,
  fullscreen_btn: Entity<FullscreenBtn>,
  exit_btn: Entity<ExitBtn>,
}

impl HeaderBar {
  pub fn new(window: &mut Window, cx: &mut App, state: Entity<BookViewerState>) -> Entity<Self> {
    let outline_btn = cx.new(|_cx| OutlineBtn::new(state.clone()));
    let scroll_mode_btn = cx.new(|_cx| ScrollModeBtn::new(state.clone()));
    let tts_btn = cx.new(|_cx| TtsBtn::new(state.clone()));

    let prev_page_btn = cx.new(|_cx| PrevPageBtn::new(state.clone()));
    let next_page_btn = cx.new(|_cx| NextPageBtn::new(state.clone()));
    let page_number_input = PageNumberInput::new(window, cx, state.clone());
    let page_counter = cx.new(|_cx| PageCounter::new(state.clone()));
    let zoom_select = ZoomSelect::new(window, cx, state.clone());

    let invert_colors_btn = cx.new(|_cx| InvertColorsBtn::new(state.clone()));
    let bookmarks_btn = cx.new(|_cx| BookmarksBtn::new(state.clone()));
    let thumbnails_btn = cx.new(|_cx| ThumbnailsBtn::new(state.clone()));
    let library_btn = cx.new(|_cx| LibraryBtn::new());
    let search_toggle_btn = cx.new(|_cx| SearchToggleBtn::new(state.clone()));
    let fullscreen_btn = cx.new(|_cx| FullscreenBtn::new(state.clone()));
    let exit_btn = cx.new(|_cx| ExitBtn::new());

    cx.new(|_cx| Self {
      outline_btn,
      scroll_mode_btn,
      tts_btn,
      prev_page_btn,
      next_page_btn,
      page_number_input,
      page_counter,
      zoom_select,
      invert_colors_btn,
      bookmarks_btn,
      thumbnails_btn,
      library_btn,
      search_toggle_btn,
      fullscreen_btn,
      exit_btn,
    })
  }
}

impl Render for HeaderBar {
  fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
    div()
      .w_full()
      .h(px(32.0))
      .bg(rgb(0x38383D))
      .flex()
      .items_center()
      .justify_between()
      .px(px(4.0))
      // Left section (3 buttons)
      .child(
        div()
          .flex()
          .items_center()
          .gap_x(px(2.0))
          .child(self.outline_btn.clone())
          .child(self.scroll_mode_btn.clone())
          .child(self.tts_btn.clone()),
      )
      // Center section
      .child(
        div()
          .flex()
          .items_center()
          .gap_x(px(4.0))
          .child(self.prev_page_btn.clone())
          .child(self.next_page_btn.clone())
          .child(self.page_number_input.clone())
          .child(self.page_counter.clone())
          .child(div().ml_1().child(self.zoom_select.clone())),
      )
      // Right section (7 buttons in strict order)
      .child(
        div()
          .flex()
          .items_center()
          .gap_x(px(2.0))
          .child(self.invert_colors_btn.clone())
          .child(self.bookmarks_btn.clone())
          .child(self.thumbnails_btn.clone())
          .child(self.library_btn.clone())
          .child(self.search_toggle_btn.clone())
          .child(self.fullscreen_btn.clone())
          .child(self.exit_btn.clone()),
      )
  }
}
