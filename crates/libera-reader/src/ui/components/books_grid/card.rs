use super::actions::{push_at_history, toggle_favorite};
use super::{
  FAVORITE_ID_PREFIX, FAVORITE_INACTIVE_OPACITY, FOOTER_HEIGHT_PX,
  TITLE_BREAKABLE_CAPACITY_MULTIPLIER, TITLE_FONT_SIZE_REM, TITLE_LINE_HEIGHT_REM,
};
use gpui::*;
use gpui_component::{ActiveTheme, Icon, IconName, Sizable, button::*};
use libera_reader_core::db::models::{CardDisplayMode, books::book::Book};
use rust_i18n::t;
use std::path::PathBuf;

impl super::BooksGrid {
  fn format_title_pixel_perfect(title: &str) -> SharedString {
    let mut breakable_title =
      String::with_capacity(title.len() * TITLE_BREAKABLE_CAPACITY_MULTIPLIER);
    for ch in title.chars() {
      breakable_title.push(ch);
      breakable_title.push('\u{200B}');
    }
    breakable_title.into()
  }

  fn get_cached_title(
    &self, book_path: &libera_reader_core::db::models::books::book::BookPath,
  ) -> SharedString {
    let mut cache = self.title_cache.borrow_mut();
    if let Some(title) = cache.get(book_path) {
      return title.clone();
    }
    let display_name = book_path.display_name();
    let new_title = Self::format_title_pixel_perfect(display_name.as_ref());
    cache.insert(book_path.clone(), new_title.clone());
    new_title
  }

  fn render_book_title(
    &self, book_path: &libera_reader_core::db::models::books::book::BookPath, foreground: Hsla,
  ) -> Div {
    let display_name = self.get_cached_title(book_path);
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
        .child(display_name),
    )
  }

  fn render_cover_click_area(
    &self, book_path: libera_reader_core::db::models::books::book::BookPath, id: SharedString,
    has_thumbnail: bool, thumbnail_path: PathBuf, cx: &Context<Self>,
  ) -> Div {
    let theme = cx.theme();
    let state_hist = self.state.clone();

    let mut cover =
      div().w_full().h_full().relative().flex().items_center().justify_center().cursor_pointer();

    if has_thumbnail {
      cover = cover.child(img(thumbnail_path).w_full().h_full().object_fit(ObjectFit::Fill));
    } else {
      cover = cover.child(
        Icon::new(IconName::BookOpen).with_size(px(40.0)).text_color(theme.foreground.opacity(0.3)),
      );
    }

    let subtle_hover = ButtonCustomVariant::new(cx)
      .hover(gpui::white().opacity(0.12))
      .active(gpui::white().opacity(0.16));
    cover.child(
      Button::new(id)
        .custom(subtle_hover)
        .xsmall()
        .absolute()
        .inset_0()
        .w_full()
        .h_full()
        .on_click(move |_ev, _window, cx| {
          push_at_history(book_path.clone(), state_hist.clone(), cx);
        }),
    )
  }

  fn render_favorite_button(
    &self, book: &Book, id: SharedString, foreground: Hsla, primary: Hsla,
  ) -> Div {
    let favorite_path = book.book_path.clone();
    let state_fav = self.state.clone();
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
            toggle_favorite(favorite_path.clone(), state_fav.clone(), cx);
          })
          .text()
          .icon(Icon::new(Icon::empty()).path(fav_icon_source).text_color(primary))
          .with_size(gpui_component::Size::Large),
      )
  }

  fn render_book_footer(&self, book: &Book, foreground: Hsla, primary: Hsla) -> Div {
    let title = self.render_book_title(&book.book_path, foreground);
    let fav_id: SharedString =
      format!("{}{}", FAVORITE_ID_PREFIX, book.book_path.full_path_string()).into();
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
      .child(self.render_favorite_button(book, fav_id, foreground, primary))
  }

  fn render_book_card_info(&self, book: &Book, cx: &Context<Self>) -> Div {
    let theme = cx.theme();
    let fav_id: SharedString =
      format!("{}{}", FAVORITE_ID_PREFIX, book.book_path.full_path_string()).into();

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
              .child(self.get_cached_title(&book.book_path)),
          )
          .child(
            div().flex().flex_col().text_sm().text_color(theme.foreground.opacity(0.7)).children([
              div().text_sm().text_color(theme.foreground.opacity(0.7)).child(format!(
                "{}: {}",
                t!("components.card.format_label"),
                book.book_path.ext.to_string().to_uppercase()
              )),
              div()
                .text_sm()
                .text_color(theme.foreground.opacity(0.7))
                .child(format!("File size: {} MB", book.book_size.as_mb())),
            ]),
          ),
      )
      .child(div().flex().justify_end().items_center().child(self.render_favorite_button(
        book,
        fav_id,
        theme.foreground,
        theme.primary,
      )))
  }

  pub(crate) fn render_list_card(
    &self, book: &Book, has_thumbnail: bool, thumbnail_path: PathBuf, card_height: Pixels,
    cx: &Context<Self>,
  ) -> Div {
    let theme = cx.theme();
    let cover_id: SharedString =
      format!("{}{}", super::COVER_ID_PREFIX, book.book_path.full_path_string()).into();

    div()
      .w_0()
      .flex_grow()
      .h(card_height)
      .flex_shrink_0()
      .flex()
      .flex_row()
      .rounded_md()
      .bg(theme.background)
      .border_1()
      .border_color(theme.border)
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
            book.book_path.clone(),
            cover_id,
            has_thumbnail,
            thumbnail_path,
            cx,
          )),
      )
      .child(self.render_book_card_info(book, cx))
  }

  pub(crate) fn render_book_card(
    &self, book: &Book, has_thumbnail: bool, thumbnail_path: PathBuf, card_height: Pixels,
    cx: &Context<Self>,
  ) -> Div {
    let mode = libera_reader_core::ctx::Ctx::global(cx).settings.read().card_display_mode;

    if mode == CardDisplayMode::List {
      return self.render_list_card(book, has_thumbnail, thumbnail_path, card_height, cx);
    }

    let theme = cx.theme();
    let cover_id: SharedString =
      format!("{}{}", super::COVER_ID_PREFIX, book.book_path.full_path_string()).into();

    let footer_height =
      if mode == CardDisplayMode::Detailed { px(FOOTER_HEIGHT_PX) } else { px(0.0) };
    let cover_height = card_height - footer_height;

    let mut card = div()
      .w_0()
      .flex_grow()
      .h(card_height)
      .flex_shrink_0()
      .flex()
      .flex_col()
      .rounded_md()
      .bg(theme.background)
      .border_1()
      .border_color(theme.border)
      .overflow_hidden()
      .child(
        div()
          .w_full()
          .h(cover_height)
          .flex_shrink_0()
          .border_b_1()
          .border_color(theme.border)
          .child(self.render_cover_click_area(
            book.book_path.clone(),
            cover_id,
            has_thumbnail,
            thumbnail_path,
            cx,
          )),
      );

    if mode == CardDisplayMode::Detailed {
      card = card.child(self.render_book_footer(book, theme.foreground, theme.primary));
    }
    card
  }
}
