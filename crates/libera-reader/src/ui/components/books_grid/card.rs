use super::FOOTER_HEIGHT_PX;
use super::actions::{book_card_click, toggle_favorite};
use crate::app_ext::AppExt;
use crate::db::models::books::book::Book;
use crate::db::models::books::book::BookPath;
use crate::ui::components::books_grid::cache::CoverState;
use gpui::*;
use gpui_component::{ActiveTheme, Icon, IconName, Sizable, button::*};
use std::path::PathBuf;

pub(crate) const FAVORITE_INACTIVE_OPACITY: f32 = 0.7;
pub(crate) const TITLE_FONT_SIZE_REM: f32 = 0.9;
pub(crate) const TITLE_LINE_HEIGHT_REM: f32 = 1.1;

impl super::BooksGrid {
  pub(crate) fn render_book_title(&self, book: &Book, foreground: Hsla) -> Div {
    div().flex_1().min_w_0().pt_1().child(
      div()
        .w_full()
        .whitespace_normal()
        .line_clamp(2)
        .text_ellipsis()
        .overflow_hidden()
        .text_size(rems(TITLE_FONT_SIZE_REM))
        .line_height(rems(TITLE_LINE_HEIGHT_REM))
        .text_color(foreground)
        .child(book.formatted_title()),
    )
  }

  /// The cover area: renders the loaded `RenderImage` (memory-safe path) or a
  /// BookOpen placeholder, with an invisible button overlay to open the book.
  pub(crate) fn render_cover_click_area(
    &self, book_path: BookPath, id: SharedString, thumb: Option<PathBuf>,
    cover_state: Option<CoverState>, cx: &Context<Self>,
  ) -> Div {
    let theme = cx.theme();

    let mut cover =
      div().w_full().h_full().relative().flex().items_center().justify_center().cursor_pointer();

    let loaded = matches!(cover_state, Some(CoverState::Loaded(_)));
    if loaded {
      if let Some(CoverState::Loaded(img)) = cover_state {
        cover = cover
          .child(gpui::img(ImageSource::Render(img)).w_full().h_full().object_fit(ObjectFit::Fill));
      }
    } else if thumb.is_some() {
      // Still loading or failed — leave the background as a placeholder box.
      cover = cover.bg(theme.background);
    } else {
      cover = cover.child(
        Icon::new(IconName::BookOpen).with_size(px(40.0)).text_color(theme.foreground.opacity(0.3)),
      );
    }

    let subtle_hover = ButtonCustomVariant::new(cx)
      .foreground(theme.foreground)
      .hover(theme.primary.opacity(0.10))
      .active(theme.primary.opacity(0.22));

    cover.child(
      Button::new(id)
        .custom(subtle_hover)
        .xsmall()
        .absolute()
        .inset_0()
        .w_full()
        .h_full()
        .on_click(move |_ev, _window, cx| {
          book_card_click(book_path.clone(), cx);
        }),
    )
  }

  pub(crate) fn render_favorite_button(
    &self, book: &Book, id: SharedString, foreground: Hsla, primary: Hsla,
  ) -> Div {
    let book_path = book.book_path.clone();
    let is_favorite = book.user_data.favorite;
    let fav_icon_source =
      if is_favorite { "heroicons--star-solid.svg" } else { "heroicons--star.svg" };

    div()
      .flex_shrink_0()
      .h_full()
      .flex()
      .items_center()
      .justify_center()
      .text_color(if is_favorite { primary } else { foreground.opacity(FAVORITE_INACTIVE_OPACITY) })
      .hover(move |h| h.text_color(primary))
      .child(
        Button::new(id)
          .ghost()
          .xsmall()
          .on_click(move |_ev, _window, cx| {
            toggle_favorite(book_path.clone(), cx);
          })
          .text()
          .icon(Icon::new(Icon::empty()).path(fav_icon_source).text_color(primary))
          .with_size(gpui_component::Size::Large),
      )
  }
  pub(crate) fn render_book_footer(&self, book: &Book, foreground: Hsla, primary: Hsla) -> Div {
    let title = self.render_book_title(book, foreground);
    div()
      .w_full()
      .h(px(FOOTER_HEIGHT_PX))
      .flex_shrink_0()
      .flex()
      .flex_row()
      .items_center()
      .justify_between()
      .gap_1()
      .px_2()
      .py_1()
      .child(title)
      .child(self.render_favorite_button(book, book.fav_btn_id(), foreground, primary))
  }

  pub(crate) fn render_book_card_info(&self, book: &Book, cx: &Context<Self>) -> Div {
    let theme = cx.theme();

    div()
      .flex_1()
      .h_full()
      .min_w_0()
      .overflow_hidden()
      .flex()
      .flex_col()
      .p_4()
      .justify_between()
      .child(
        div()
          .flex()
          .flex_col()
          .min_w_0()
          .child(
            div()
              .text_base()
              .line_height(rems(1.2))
              .font_weight(FontWeight::BOLD)
              .text_color(theme.foreground)
              .whitespace_normal()
              .overflow_hidden()
              .text_ellipsis()
              .line_clamp(3)
              .child(book.formatted_title()),
          )
          .child(
            div().flex().flex_col().text_sm().text_color(theme.foreground.opacity(0.7)).children([
              div().text_sm().text_color(theme.foreground.opacity(0.7)).child(book.format_label()),
              div().text_sm().text_color(theme.foreground.opacity(0.7)).child(book.size_label()),
            ]),
          ),
      )
      .child(div().flex().justify_end().items_center().child(self.render_favorite_button(
        book,
        book.fav_btn_id(),
        theme.foreground,
        theme.primary,
      )))
  }

  pub(crate) fn render_list_card(
    &self, book: &Book, thumb: Option<PathBuf>, cover_state: Option<CoverState>,
    card_height: Pixels, cx: &Context<Self>,
  ) -> Div {
    let theme = cx.theme();
    let book_path = book.book_path.clone();

    div()
      .w_0()
      .flex_grow_1()
      .h(card_height)
      .flex_shrink_0()
      .flex()
      .flex_row()
      .rounded_md()
      .bg(theme.background)
      .border_2()
      .border_color(theme.border)
      .hover(move |s| s.border_color(theme.primary).bg(theme.foreground.opacity(0.03)).shadow_lg())
      .cursor_pointer()
      .overflow_hidden()
      .child(
        div()
          .flex_1()
          .h_full()
          .min_w_0()
          .border_r_1()
          .border_color(theme.border)
          .overflow_hidden()
          .child(self.render_cover_click_area(
            book_path,
            book.cover_btn_id(),
            thumb,
            cover_state,
            cx,
          )),
      )
      .child(self.render_book_card_info(book, cx))
  }

  pub(crate) fn render_book_card_detailed_or_compact(
    &self, book: &Book, thumb: Option<PathBuf>, cover_state: Option<CoverState>,
    card_height: Pixels, cx: &Context<Self>,
  ) -> Div {
    let mode = cx.settings().read().card_display_mode;
    let theme = cx.theme();
    let book_path = book.book_path.clone();

    let footer_height = if mode == crate::db::models::CardDisplayMode::Detailed {
      px(FOOTER_HEIGHT_PX)
    } else {
      px(0.0)
    };
    let cover_height = card_height - footer_height;

    let mut card = div()
      .w_0()
      .flex_grow_1()
      .h(card_height)
      .flex_shrink_0()
      .flex()
      .flex_col()
      .rounded_md()
      .bg(theme.background)
      .border_2()
      .border_color(theme.border)
      .hover(move |s| s.border_color(theme.primary).shadow_lg())
      .cursor_pointer()
      .overflow_hidden()
      .child(
        div()
          .w_full()
          .h(cover_height)
          .flex_shrink_0()
          .border_b_1()
          .border_color(theme.border)
          .child(self.render_cover_click_area(
            book_path,
            book.cover_btn_id(),
            thumb,
            cover_state,
            cx,
          )),
      );

    if mode == crate::db::models::CardDisplayMode::Detailed {
      card = card.child(self.render_book_footer(book, theme.foreground, theme.primary));
    }
    card
  }
}

/// Free function entry point used by the grid render loop.
pub(crate) fn render_book_card(
  view: &mut super::BooksGrid, book: &Book, thumb: Option<PathBuf>,
  cover_state: Option<CoverState>, card_height: Pixels, cx: &Context<super::BooksGrid>,
) -> Div {
  let mode = cx.settings().read().card_display_mode;
  if mode == crate::db::models::CardDisplayMode::List {
    view.render_list_card(book, thumb, cover_state, card_height, cx)
  } else {
    view.render_book_card_detailed_or_compact(book, thumb, cover_state, card_height, cx)
  }
}
