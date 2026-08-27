use crate::ui::pages::book_viewer::state::{BookViewerState, ZoomPreset};
use gpui::*;
use gpui_component::{
  IndexPath, Sizable,
  select::{SearchableVec, Select, SelectEvent, SelectState},
};

pub struct ZoomSelect {
  state: Entity<BookViewerState>,
  select_state: Entity<SelectState<SearchableVec<ZoomPreset>>>,
  _subscription: Subscription,
}

impl ZoomSelect {
  pub fn new(window: &mut Window, cx: &mut App, state: Entity<BookViewerState>) -> Entity<Self> {
    let presets = ZoomPreset::all();
    let current = state.read(cx).zoom_preset;
    let initial_idx =
      presets.iter().position(|p| *p == current).map(|i| IndexPath::default().row(i));

    let searchable = SearchableVec::new(presets);
    let select_state =
      cx.new(|cx| SelectState::new(searchable, initial_idx, window, cx).searchable(false));

    let state_clone = state.clone();
    cx.new(|cx: &mut Context<Self>| {
      let subscription = cx.subscribe_in(&select_state, window, {
        let state = state_clone.clone();
        move |_this: &mut Self,
              _select: &Entity<SelectState<SearchableVec<ZoomPreset>>>,
              event: &SelectEvent<SearchableVec<ZoomPreset>>,
              _window: &mut Window,
              cx: &mut Context<'_, Self>| {
          if let SelectEvent::Confirm(Some(preset)) = event {
            state.update(cx, |s, cx| {
              s.set_zoom(*preset);
              cx.notify();
            });
          }
        }
      });

      Self { state, select_state, _subscription: subscription }
    })
  }
}

impl Render for ZoomSelect {
  fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
    div()
      .w(px(110.0))
      .h(px(28.0))
      .rounded(px(3.0))
      .border_1()
      .border_color(rgb(0xB3B4B7))
      .bg(rgb(0x4A4A4F))
      .hover(|s| s.bg(rgb(0x666667)))
      .flex()
      .items_center()
      .child(Select::new(&self.select_state).small().bg(gpui::transparent_black()).w_full())
  }
}
