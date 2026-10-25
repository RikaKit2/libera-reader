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
  _state_subscription: Subscription,
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
                let available_width =
                  if sidebar_open { window_size.width - px(250.0) } else { window_size.width };
                let available_height = window_size.height - px(32.0);
                s.update_fit_zoom(size(available_width, available_height));
              }
              cx.notify();
            });
          }
        }
      });
      let state_subscription = cx.observe(&state, |_this, _state, cx| {
        cx.notify();
      });

      Self {
        state,
        select_state,
        _subscription: subscription,
        _state_subscription: state_subscription,
      }
    })
  }
}

impl Render for ZoomSelect {
  fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let (zoom_preset, zoom_factor) = {
      let s = self.state.read(cx);
      (s.zoom_preset, s.zoom_factor)
    };
    let presets = ZoomPreset::all();
    let matching_preset_idx = match zoom_preset {
      ZoomPreset::FitWidth => presets.iter().position(|p| *p == ZoomPreset::FitWidth),
      ZoomPreset::FitPage => presets.iter().position(|p| *p == ZoomPreset::FitPage),
      _ => presets.iter().position(|p| p.factor().is_some_and(|f| (f - zoom_factor).abs() < 0.01)),
    };

    let target_index = matching_preset_idx.map(|row| IndexPath::default().row(row));
    let current_selected = self.select_state.read(cx).selected_index(cx);

    if current_selected != target_index {
      self.select_state.update(cx, |state, cx| {
        state.set_selected_index(target_index, window, cx);
      });
    }

    let percent_label = format!("{}%", (zoom_factor * 100.0).round() as u32);

    div()
      .id("header-zoom-select")
      .w(header::ZOOM_SELECT_WIDTH)
      .child(Select::new(&self.select_state).placeholder(percent_label).small().w_full())
  }
}
