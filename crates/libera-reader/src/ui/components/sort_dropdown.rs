use gpui::*;
use gpui::{App, Context, Entity, IntoElement, ParentElement, Render, Styled, Window, px};
use gpui_component::button::ButtonVariants;
use gpui_component::{button::Button, select::*, *};

use crate::books_state::{BooksState, SortField, TargetList};

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
    };

    let fields = SearchableVec::new(SortField::all().to_vec());
    let initial_index =
      SortField::all().iter().position(|f| *f == current_field).map(|i| IndexPath::default().row(i));

    let sort_select = cx.new(|cx| SelectState::new(fields, initial_index, window, cx).searchable(false));

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
    };
    let current_index =
      SortField::all().iter().position(|f| *f == current_field).map(|i| IndexPath::default().row(i));
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
    };

    let icon_name = if is_reversed { IconName::SortDescending } else { IconName::SortAscending };

    div().flex().gap_x_2().mr_2().items_center().children([
      div().child(Select::new(&self.sort_select).w(px(140.0))),
      div().child(Button::new("reverse_btn").text().icon(Icon::new(icon_name).small()).on_click({
        let books_state = self.books_state.clone();
        let target = self.target;
        move |_ev, _window, cx| {
          books_state.update(cx, |state, cx| {
            state.toggle_reverse(target, cx);
          });
        }
      })),
    ])
  }
}
