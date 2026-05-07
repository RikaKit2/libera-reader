use gpui::*;
use gpui::{App, Context, Entity, IntoElement, ParentElement, Render, Styled, Window, px};
use gpui_component::{Icon, button::Button, select::*, *};

use crate::books_state::{BooksState, SortField, TargetList};
use libera_reader_core::ctx::Ctx;
use libera_reader_core::db::models::CardDisplayMode;

pub struct SortControls {
  books_state: Entity<BooksState>,
  target: TargetList,
  sort_select: Entity<SelectState<SearchableVec<SortField>>>,
}

impl SortControls {
  pub fn new(
    window: &mut Window, cx: &mut App, books_state: Entity<BooksState>, target: TargetList,
  ) -> Entity<Self> {
    let current_field = match target {
      TargetList::Library => books_state.read(cx).library_sort.field,
      TargetList::Favorites => books_state.read(cx).favorites_sort.field,
      TargetList::History => books_state.read(cx).history_sort.field,
      TargetList::Bookmarks => books_state.read(cx).bookmarks_sort.field,
    };

    let available_fields = SortField::available_for(target);
    let fields = SearchableVec::new(available_fields.clone());
    let initial_index = available_fields
      .iter()
      .position(|f| *f == current_field)
      .map(|i| IndexPath::default().row(i));

    let sort_select =
      cx.new(|cx| SelectState::new(fields, initial_index, window, cx).searchable(false));

    cx.new(|cx| {
      cx.subscribe_in(&sort_select, window, {
        let books_state = books_state.clone();
        move |_this,
              _state: &Entity<SelectState<SearchableVec<SortField>>>,
              event: &SelectEvent<SearchableVec<SortField>>,
              _window,
              cx| {
          let SelectEvent::Confirm(new_field) = event;
          if let Some(new_field) = new_field {
            books_state.update(cx, |state, cx| {
              state.set_sort_field(*new_field, target, cx);
            });
          }
        }
      })
      .detach();

      Self { books_state, target, sort_select }
    })
  }

  fn sync_select(&self, window: &mut Window, cx: &mut Context<Self>) {
    let current_field = match self.target {
      TargetList::Library => self.books_state.read(cx).library_sort.field,
      TargetList::Favorites => self.books_state.read(cx).favorites_sort.field,
      TargetList::History => self.books_state.read(cx).history_sort.field,
      TargetList::Bookmarks => self.books_state.read(cx).bookmarks_sort.field,
    };
    let available_fields = SortField::available_for(self.target);
    let current_index = available_fields
      .iter()
      .position(|f| *f == current_field)
      .map(|i| IndexPath::default().row(i));
    if let Some(idx) = current_index {
      self.sort_select.update(cx, |s, cx| {
        s.set_selected_index(Some(idx), window, cx);
      });
    }
  }
}

impl Render for SortControls {
  fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    self.sync_select(window, cx);

    let is_reversed = match self.target {
      TargetList::Library => self.books_state.read(cx).library_sort.is_reversed,
      TargetList::Favorites => self.books_state.read(cx).favorites_sort.is_reversed,
      TargetList::History => self.books_state.read(cx).history_sort.is_reversed,
      TargetList::Bookmarks => self.books_state.read(cx).bookmarks_sort.is_reversed,
    };

    let dir_icon_path = if is_reversed { "a-arrow-down.svg" } else { "a-arrow-up.svg" };

    let display_mode = Ctx::global(cx).settings.read().card_display_mode;
    let mode_icon = match display_mode {
      CardDisplayMode::Compact => "layout-grid.svg",
      CardDisplayMode::Detailed => "layout-dashboard.svg",
      CardDisplayMode::List => "layout-list.svg",
    };

    let view_entity = cx.entity().clone();

    div().flex().gap_x_2().mr_2().items_center().children([
      div().child(Select::new(&self.sort_select).w(px(190.0))),
      div().child(
        Button::new("reverse_btn")
          .icon(Icon::new(Icon::empty()).path(dir_icon_path).with_size(px(18.0)))
          .on_click({
            let books_state = self.books_state.clone();
            let target = self.target;
            move |_ev, _window, cx| {
              books_state.update(cx, |state, cx| {
                state.toggle_reverse(target, cx);
              });
            }
          }),
      ),
      div().child(
        Button::new("toggle_display_mode")
          .icon(Icon::new(Icon::empty()).path(mode_icon).with_size(px(18.0)))
          .on_click(move |_ev, _window, cx| {
            let current = Ctx::global(cx).settings.read().card_display_mode;
            let new_mode = match current {
              CardDisplayMode::Compact => CardDisplayMode::Detailed,
              CardDisplayMode::Detailed => CardDisplayMode::List,
              CardDisplayMode::List => CardDisplayMode::Compact,
            };
            let _ = Ctx::global_mut(cx).settings.set_display_mode(new_mode);
            cx.notify(view_entity.entity_id());
          }),
      ),
    ])
  }
}
