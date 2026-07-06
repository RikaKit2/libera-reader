use gpui::*;
use gpui_component::{button::*, select::*, *};

use crate::books_state::models::DisplayModeOption;
use crate::books_state::{BooksState, SortField, TargetList};
use libera_reader_core::ctx::Ctx;

#[allow(dead_code)]
pub struct SortControls {
  books_state: Entity<BooksState>,
  target: TargetList,
  sort_select: Entity<SelectState<SearchableVec<SortField>>>,
  mode_select: Entity<SelectState<SearchableVec<DisplayModeOption>>>,
}

impl SortControls {
  #[allow(dead_code)]
  pub fn new(
    window: &mut Window, cx: &mut App, books_state: Entity<BooksState>, target: TargetList,
  ) -> Entity<Self> {
    let current_field = match target {
      TargetList::Library => books_state.read(cx).library_sort.field,
      TargetList::Favorites => books_state.read(cx).favorites_sort.field,
      TargetList::History => books_state.read(cx).history_sort.field,
      TargetList::Bookmarks => books_state.read(cx).bookmarks_sort.field,
    };
    let current_mode = Ctx::global(cx).settings.read().card_display_mode;

    let available_fields = SortField::available_for(target);
    let fields = SearchableVec::new(available_fields.clone());
    let sort_idx = available_fields
      .iter()
      .position(|f| *f == current_field)
      .map(|i| IndexPath::default().row(i));
    let sort_select = cx.new(|cx| SelectState::new(fields, sort_idx, window, cx).searchable(false));

    let modes = DisplayModeOption::all();
    let modes_vec = SearchableVec::new(modes.clone());
    let mode_idx =
      modes.iter().position(|m| m.value() == &current_mode).map(|i| IndexPath::default().row(i));
    let mode_select =
      cx.new(|cx| SelectState::new(modes_vec, mode_idx, window, cx).searchable(false));

    cx.new(|cx| {
      cx.subscribe_in(&sort_select, window, {
        let books_state = books_state.clone();
        move |_this: &mut SortControls,
              _state: &Entity<SelectState<SearchableVec<SortField>>>,
              event: &SelectEvent<SearchableVec<SortField>>,
              _window: &mut Window,
              cx: &mut Context<'_, SortControls>| {
          if let SelectEvent::Confirm(Some(new_field)) = event {
            books_state.update(cx, |state, cx| state.set_sort_field(*new_field, target, cx));
          }
        }
      })
      .detach();

      cx.subscribe_in(&mode_select, window, {
        move |_this: &mut SortControls,
              _state: &Entity<SelectState<SearchableVec<DisplayModeOption>>>,
              event: &SelectEvent<SearchableVec<DisplayModeOption>>,
              _window: &mut Window,
              cx: &mut Context<'_, SortControls>| {
          if let SelectEvent::Confirm(Some(new_mode)) = event {
            let _ = Ctx::global_mut(cx).settings.set_display_mode(*new_mode);
            cx.notify();
          }
        }
      })
      .detach();

      Self { books_state, target, sort_select, mode_select }
    })
  }
}

impl Render for SortControls {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let is_reversed = match self.target {
      TargetList::Library => self.books_state.read(cx).library_sort.is_reversed,
      TargetList::Favorites => self.books_state.read(cx).favorites_sort.is_reversed,
      TargetList::History => self.books_state.read(cx).history_sort.is_reversed,
      TargetList::Bookmarks => self.books_state.read(cx).bookmarks_sort.is_reversed,
    };

    let dir_icon = if is_reversed { "a-arrow-down.svg" } else { "a-arrow-up.svg" };

    div().flex().gap_x_2().mr_2().items_center().children([
      div().child(Select::new(&self.sort_select).w(px(140.0))),
      div().child({
        let subtle_hover = ButtonCustomVariant::new(cx)
          .color(cx.theme().background)
          .hover(cx.theme().background.opacity(0.5));

        Button::new("reverse_btn")
          .icon(Icon::new(Icon::empty()).path(dir_icon))
          .custom(subtle_hover)
          .on_click({
            let books_state = self.books_state.clone();
            let target = self.target;
            move |_ev, _window, cx| {
              books_state.update(cx, |state, cx| state.toggle_reverse(target, cx));
            }
          })
      }),
      div().child(Select::new(&self.mode_select).w(px(64.0)).menu_width(px(64.0))),
    ])
  }
}
