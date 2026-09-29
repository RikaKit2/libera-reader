use crate::ui::pages::book_viewer::constants::header;
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
              if matches!(*preset, ZoomPreset::FitWidth | ZoomPreset::FitPage) {
                let sidebar_open = s.active_sidebar_tab.is_open();
                let window_size = _window.viewport_size();
                let available_width = if sidebar_open {
                  window_size.width - px(250.0)
                } else {
                  window_size.width
                };
                let available_height = window_size.height - px(32.0);
                s.update_fit_zoom(size(available_width, available_height));
              }
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
      .id("header-zoom-select")
      .w(header::ZOOM_SELECT_WIDTH)
      .child(Select::new(&self.select_state).small().w_full())
  }
}
