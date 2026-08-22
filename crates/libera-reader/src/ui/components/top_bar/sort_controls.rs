use gpui::*;
use gpui_component::{ActiveTheme, select::*, *};

use crate::app_ext::AppExt;
use crate::books_state::models::DisplayModeOption;
use crate::books_state::{BooksState, SortField, TargetList};
use crate::ui::components::top_bar::reverse_btn::ReverseBtn;

/// Sort controls for the top bar: sort-field select, reverse-direction button
/// (its own component), and display-mode select.
pub struct SortControls {
  sort_select: Entity<SelectState<SearchableVec<SortField>>>,
  mode_select: Entity<SelectState<SearchableVec<DisplayModeOption>>>,
  reverse_btn: Entity<ReverseBtn>,
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
    let current_mode = cx.settings().read().card_display_mode;

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

    let reverse_btn = ReverseBtn::new(books_state.clone(), target, cx);

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
            let _ = cx.settings_mut().set_display_mode(*new_mode);
            cx.notify();
          }
        }
      })
      .detach();

      Self { sort_select, mode_select, reverse_btn }
    })
  }
}

impl Render for SortControls {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let bg = cx.theme().background;

    div().flex().gap_x_2().mr_2().items_center().children([
      div().child(Select::new(&self.sort_select).bg(bg).w(px(140.0))),
      div().child(self.reverse_btn.clone()),
      div().child(Select::new(&self.mode_select).bg(bg).w(px(64.0)).menu_width(px(64.0))),
    ])
  }
}
