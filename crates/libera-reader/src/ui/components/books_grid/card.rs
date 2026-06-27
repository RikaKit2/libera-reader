use super::actions::{push_at_history, toggle_favorite};
use super::{
  FAVORITE_ID_PREFIX, FAVORITE_INACTIVE_OPACITY, FOOTER_HEIGHT_PX, TITLE_FONT_SIZE_REM,
  TITLE_LINE_HEIGHT_REM,
};
use crate::books_state::LightBook;
use gpui::*;
use gpui_component::{ActiveTheme, Icon, IconName, Sizable, button::*};
use libera_reader_core::ctx::Ctx;
use libera_reader_core::db::models::CardDisplayMode;
use libera_reader_core::db::models::books::book::BookPath;
use rust_i18n::t;

impl super::BooksGrid {
  fn render_book_title(&self, book: &LightBook, foreground: Hsla) -> Div {
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
        .child(book.formatted_title.clone()),
    )
  }

  fn render_cover_click_area(&self, book: &LightBook, id: SharedString, cx: &Context<Self>) -> Div {
    let theme = cx.theme();
    let state_hist = self.state.clone();
    let id_for_click: SharedString = book.id.clone();

    let mut cover =
      div().w_full().h_full().relative().flex().items_center().justify_center().cursor_pointer();

    let db = Ctx::global(cx).db.clone();
    let mut img_data = None;
    if book.has_thumbnail
      && let Ok(mut cache) = self.image_cache.try_borrow_mut()
    {
      img_data = cache.get(&book.id, &db);
    }

    let has_thumbnail = book.has_thumbnail;
    let failed_to_load = has_thumbnail && img_data.is_none();

    if book.size == 0 {
      // Render a clear 0-byte warning instead of a blank card or book open icon
      cover = cover.child(
        div()
          .flex()
          .flex_col()
          .items_center()
          .justify_center()
          .gap_1()
          .child(Icon::new(IconName::TriangleAlert).with_size(px(32.0)).text_color(gpui::red()))
          .child(
            div().text_xs().font_weight(FontWeight::BOLD).text_color(gpui::red()).child("0 BYTES"),
          ),
      );
    } else {
      let mut mutool_err = None;
      if let Ok(Some(err)) = book.get_mutool_error(&db) {
        mutool_err = Some(err);
      }

      if let Some(err) = mutool_err {
        // Render a clear mutool extraction error on the card
        cover = cover.child(
          div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap_1()
            .child(Icon::new(IconName::TriangleAlert).with_size(px(32.0)).text_color(gpui::red()))
            .child(
              div()
                .text_xs()
                .font_weight(FontWeight::BOLD)
                .text_color(gpui::red())
                .child(format!("{:?}", err).to_uppercase()),
            ),
        );
      } else if has_thumbnail {
        if let Some(img_data) = img_data {
          cover = cover.child(img(img_data).w_full().h_full().object_fit(ObjectFit::Fill));
        } else {
          cover = cover.child(
            Icon::new(IconName::BookOpen)
              .with_size(px(40.0))
              .text_color(theme.foreground.opacity(0.3)),
          );
        }
      } else {
        cover = cover.child(
          Icon::new(IconName::BookOpen)
            .with_size(px(40.0))
            .text_color(theme.foreground.opacity(0.3)),
        );
      }
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
          // If the book should have a thumbnail but it failed to load, re-queue it for extraction on click
          if failed_to_load {
            // Re-queue for extraction
            let not_cached = Ctx::global(cx).not_cached_books.clone();
            let _ = not_cached.tx().send(BookPath::from_id(&id_for_click));
          }
          push_at_history(BookPath::from_id(&id_for_click), state_hist.clone(), cx);
        }),
    )
  }

  fn render_favorite_button(
    &self, book: &LightBook, id: SharedString, foreground: Hsla, primary: Hsla,
  ) -> Div {
    let id_for_fav: SharedString = book.id.clone();
    let state_fav = self.state.clone();
    let is_favorite = book.is_favorite;
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
            let path = BookPath::from_id(&id_for_fav);
            toggle_favorite(path, state_fav.clone(), cx);
          })
          .text()
          .icon(Icon::new(Icon::empty()).path(fav_icon_source).text_color(primary))
          .with_size(gpui_component::Size::Large),
      )
  }

  fn render_book_footer(&self, book: &LightBook, foreground: Hsla, primary: Hsla) -> Div {
    let title = self.render_book_title(book, foreground);
    let fav_id: SharedString = format!("{}{}", FAVORITE_ID_PREFIX, book.id).into();
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

  fn render_book_card_info(&self, book: &LightBook, cx: &Context<Self>) -> Div {
    let theme = cx.theme();
    let fav_id: SharedString = format!("{}{}", FAVORITE_ID_PREFIX, book.id).into();

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
              .child(book.formatted_title.clone()),
          )
          .child(
            div().flex().flex_col().text_sm().text_color(theme.foreground.opacity(0.7)).children([
              div().text_sm().text_color(theme.foreground.opacity(0.7)).child(format!(
                "{}: {}",
                t!("components.card.format_label"),
                book.ext.to_uppercase()
              )),
              div()
                .text_sm()
                .text_color(theme.foreground.opacity(0.7))
                .child(format!("File size: {} MB", book.size / (1024 * 1024))),
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
    &self, book: &LightBook, card_height: Pixels, cx: &Context<Self>,
  ) -> Div {
    let theme = cx.theme();
    let cover_id: SharedString = format!("{}{}", super::COVER_ID_PREFIX, book.id).into();

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
          .child(self.render_cover_click_area(book, cover_id, cx)),
      )
      .child(self.render_book_card_info(book, cx))
  }

  pub(crate) fn render_book_card(
    &self, book: &LightBook, card_height: Pixels, cx: &Context<Self>,
  ) -> Div {
    let mode = libera_reader_core::ctx::Ctx::global(cx).settings.read().card_display_mode;

    if mode == CardDisplayMode::List {
      return self.render_list_card(book, card_height, cx);
    }

    let theme = cx.theme();
    let cover_id: SharedString = format!("{}{}", super::COVER_ID_PREFIX, book.id).into();

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
          .child(self.render_cover_click_area(book, cover_id, cx)),
      );

    if mode == CardDisplayMode::Detailed {
      card = card.child(self.render_book_footer(book, theme.foreground, theme.primary));
    }
    card
  }
}
