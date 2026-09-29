#![allow(dead_code)]

pub mod actions;

pub mod cache;
pub mod constants;
pub mod document;
pub mod header;
pub mod image_utils;
pub mod loader;
pub mod sidebar;
pub mod state;
pub mod viewport;

use crate::ui::pages::book_viewer::state::{SidebarTab, ZoomPreset};
pub use actions::*;
pub use cache::{BookViewerCache, PageImageState, PageLinksState, PageTextState};
pub use document::DocumentData;
pub use header::Header;
pub use sidebar::SideBar;
pub use state::BookViewerState;
pub use viewport::Viewport;

use crate::app_ext::AppExt;
use gpui::prelude::*;
use gpui::{App, Context, Entity, FocusHandle, IntoElement, MouseButton, ParentElement, Render, Styled, Subscription, Window, div, px, size};
use crate::db::models::settings::{RootRoute, Route};

pub(crate) struct BookViewer {
  state: Entity<BookViewerState>,
  header: Entity<Header>,
  sidebar: Entity<SideBar>,
  viewport: Entity<Viewport>,
  focus_handle: FocusHandle,
  _window_subscription: Subscription,
}

impl BookViewer {
  pub(crate) fn new(window: &mut Window, cx: &mut App) -> Entity<Self> {
    let state = cx.book_viewer_state().clone();
    let header = Header::new(window, cx, state.clone());
    let sidebar = SideBar::new(window, cx, state.clone());
    let viewport = Viewport::new(state.clone(), cx);

    cx.bind_keys(actions::default_keybindings());

    cx.new(|cx| {
      let focus_handle = cx.focus_handle();
      window.focus(&focus_handle, cx);

      let sub = cx.observe_window_bounds(window, |this: &mut Self, window, cx| {
        let sidebar_open = this.state.read(cx).active_sidebar_tab.is_open();
        let window_size = window.viewport_size();
        let available_width = if sidebar_open {
          window_size.width - px(250.0)
        } else {
          window_size.width
        };
        let available_height = window_size.height - px(32.0);
        this.state.update(cx, |s, cx| {
          if matches!(s.zoom_preset, ZoomPreset::FitWidth | ZoomPreset::FitPage) {
            s.update_fit_zoom(size(available_width, available_height));
            cx.notify();
          }
        });
      });

      Self {
        state,
        header,
        sidebar,
        viewport,
        focus_handle,
        _window_subscription: sub,
      }
    })
  }

  pub(crate) fn state(&self) -> &Entity<BookViewerState> {
    &self.state
  }
}

impl Render for BookViewer {
  fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    if !self.focus_handle.is_focused(window) {
      window.focus(&self.focus_handle, cx);
    }

    div()
      .id("book-viewer-root")
      .track_focus(&self.focus_handle)
      .key_context(actions::BOOK_VIEWER_CONTEXT)
      .on_mouse_down(
        MouseButton::Left,
        cx.listener(|this, _, window, cx| {
          window.focus(&this.focus_handle, cx);
        }),
      )
      .on_action(cx.listener(|this, _: &actions::NextPage, _window, cx| {
        this.state.update(cx, |s, cx| {
          s.next_page();
          cx.notify();
        });
      }))
      .on_action(cx.listener(|this, _: &actions::PrevPage, _window, cx| {
        this.state.update(cx, |s, cx| {
          s.prev_page();
          cx.notify();
        });
      }))
      .on_action(cx.listener(|this, _: &actions::FirstPage, _window, cx| {
        this.state.update(cx, |s, cx| {
          s.go_to_page(1);
          cx.notify();
        });
      }))
      .on_action(cx.listener(|this, _: &actions::LastPage, _window, cx| {
        this.state.update(cx, |s, cx| {
          let total = s.total_pages;
          s.go_to_page(total);
          cx.notify();
        });
      }))
      .on_action(cx.listener(|this, _: &actions::ZoomIn, _window, cx| {
        this.state.update(cx, |s, cx| {
          s.zoom_in();
          cx.notify();
        });
      }))
      .on_action(cx.listener(|this, _: &actions::ZoomOut, _window, cx| {
        this.state.update(cx, |s, cx| {
          s.zoom_out();
          cx.notify();
        });
      }))
      .on_action(cx.listener(|this, _: &actions::ResetZoom, _window, cx| {
        this.state.update(cx, |s, cx| {
          s.set_zoom(ZoomPreset::Percent100);
          cx.notify();
        });
      }))
      .on_action(cx.listener(|this, _: &actions::FitWidth, window, cx| {
        this.state.update(cx, |s, cx| {
          s.set_zoom(ZoomPreset::FitWidth);
          let sidebar_open = s.active_sidebar_tab.is_open();
          let window_size = window.viewport_size();
          let available_width = if sidebar_open {
            window_size.width - px(250.0)
          } else {
            window_size.width
          };
          let available_height = window_size.height - px(32.0);
          s.update_fit_zoom(size(available_width, available_height));
          cx.notify();
        });
      }))
      .on_action(cx.listener(|this, _: &actions::FitPage, window, cx| {
        this.state.update(cx, |s, cx| {
          s.set_zoom(ZoomPreset::FitPage);
          let sidebar_open = s.active_sidebar_tab.is_open();
          let window_size = window.viewport_size();
          let available_width = if sidebar_open {
            window_size.width - px(250.0)
          } else {
            window_size.width
          };
          let available_height = window_size.height - px(32.0);
          s.update_fit_zoom(size(available_width, available_height));
          cx.notify();
        });
      }))
      .on_action(cx.listener(|this, _: &actions::ToggleSearch, _window, cx| {
        this.state.update(cx, |s, cx| {
          s.toggle_search();
          cx.notify();
        });
      }))
      .on_action(cx.listener(|this, _: &actions::NextSearchMatch, _window, cx| {
        this.state.update(cx, |s, cx| {
          s.next_search_match();
          cx.notify();
        });
      }))
      .on_action(cx.listener(|this, _: &actions::PrevSearchMatch, _window, cx| {
        this.state.update(cx, |s, cx| {
          s.prev_search_match();
          cx.notify();
        });
      }))
      .on_action(cx.listener(|this, _: &actions::CloseSearch, _window, cx| {
        this.state.update(cx, |s, cx| {
          if s.search_open {
            s.toggle_search();
            cx.notify();
          } else {
            let _ = cx.settings_mut().set_route(RootRoute::Main(Route::Library));
          }
        });
      }))
      .on_action(cx.listener(|_this, _: &actions::ExitViewer, _window, cx| {
        let _ = cx.settings_mut().set_route(RootRoute::Main(Route::Library));
      }))
      .on_action(cx.listener(|this, _: &actions::ToggleFullscreen, window, cx| {
        this.state.update(cx, |s, _cx| {
          s.toggle_fullscreen(window);
        });
      }))
      .on_action(cx.listener(|this, _: &actions::ToggleInvertColors, _window, cx| {
        this.state.update(cx, |s, cx| {
          s.toggle_invert_colors();
          cx.notify();
        });
      }))
      .on_action(cx.listener(|this, _: &actions::ToggleBookmarks, _window, cx| {
        this.state.update(cx, |s, cx| {
          s.toggle_sidebar_tab(SidebarTab::Bookmarks);
          cx.notify();
        });
      }))
      .on_action(cx.listener(|this, _: &actions::ToggleOutline, _window, cx| {
        this.state.update(cx, |s, cx| {
          s.toggle_sidebar_tab(SidebarTab::Outline);
          cx.notify();
        });
      }))
      .on_action(cx.listener(|this, _: &actions::ToggleThumbnails, _window, cx| {
        this.state.update(cx, |s, cx| {
          s.toggle_sidebar_tab(SidebarTab::Thumbnails);
          cx.notify();
        });
      }))
      .on_action(cx.listener(|this, _: &actions::ToggleTts, _window, cx| {
        this.state.update(cx, |s, cx| {
          s.toggle_sidebar_tab(SidebarTab::Tts);
          cx.notify();
        });
      }))
      .size_full()
      .flex()
      .flex_col()
      .child(self.header.clone())
      .child(
        div()
          .flex_1()
          .flex()
          .w_full()
          .overflow_hidden()
          .child(self.sidebar.clone())
          .child(self.viewport.clone()),
      )
  }
}
