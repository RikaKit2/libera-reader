use gpui::{
  AppContext, AsyncApp, Context, Entity, IntoElement, ParentElement, Render, SharedString, Styled,
  Window, div, px,
};
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::input::*;
use gpui_component::{ActiveTheme, Disableable, Sizable, gray, h_flex};
use rfd::AsyncFileDialog;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

static DIALOG_OPEN: AtomicBool = AtomicBool::new(false);

const CACHE_SIZE_MIN: usize = 10;
const CACHE_SIZE_MAX: usize = 100000;
const CACHE_SIZE_STEP: usize = 10;

#[derive(Clone)]
pub struct SelectedFolder(pub PathBuf);
impl gpui::Global for SelectedFolder {}

#[derive(Clone)]
pub struct CacheSize(pub usize);
impl gpui::Global for CacheSize {}

pub struct FolderSelectPage {
  cache_size: usize,
  cache_input: Option<Entity<InputState>>,
}

impl FolderSelectPage {
  pub fn new() -> Self {
    Self { cache_size: 10000, cache_input: None }
  }
}

impl Render for FolderSelectPage {
  fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    // Lazily create InputState on first render (Window is available now)
    if self.cache_input.is_none() {
      let input = cx.new(|cx| {
        InputState::new(window, cx).default_value(self.cache_size.to_string()).placeholder("10000")
      });

      cx.subscribe_in(&input, window, {
        move |_this: &mut FolderSelectPage,
              state: &Entity<InputState>,
              event: &InputEvent,
              _window: &mut Window,
              cx: &mut Context<'_, FolderSelectPage>| {
          if let InputEvent::Change = event {
            let text = state.read(cx).value();
            if let Ok(val) = text.parse::<usize>() {
              let clamped = val.clamp(CACHE_SIZE_MIN, CACHE_SIZE_MAX);
              _this.cache_size = clamped;
              cx.set_global(CacheSize(clamped));
            }
          }
        }
      })
      .detach();

      cx.subscribe_in(&input, window, {
        move |_this: &mut FolderSelectPage,
              state: &Entity<InputState>,
              event: &NumberInputEvent,
              _window: &mut Window,
              cx: &mut Context<'_, FolderSelectPage>| {
          let NumberInputEvent::Step(action) = event;
          let new_val = match action {
            StepAction::Increment => (_this.cache_size + CACHE_SIZE_STEP).min(CACHE_SIZE_MAX),
            StepAction::Decrement => {
              _this.cache_size.saturating_sub(CACHE_SIZE_STEP).max(CACHE_SIZE_MIN)
            }
          };
          if new_val != _this.cache_size {
            _this.cache_size = new_val;
            cx.set_global(CacheSize(new_val));
            state.update(cx, |s, cx| {
              s.set_value(new_val.to_string(), _window, cx);
            });
          }
        }
      })
      .detach();

      self.cache_input = Some(input);
    }

    let dialog_open = DIALOG_OPEN.load(Ordering::Relaxed);
    let path_display: SharedString = cx
      .try_global::<SelectedFolder>()
      .map(|g| g.0.to_string_lossy().to_string().into())
      .unwrap_or_else(|| SharedString::from("No folder selected"));

    let header = div().child("Select a folder with images:");
    let picker = div().child(
      Button::new("pick-folder")
        .primary()
        .label("Choose Folder")
        .disabled(dialog_open)
        .w(px(220.0))
        .on_click(cx.listener(|_this, _event, _window, cx| {
          if DIALOG_OPEN.load(Ordering::Relaxed) {
            return;
          }
          DIALOG_OPEN.store(true, Ordering::Relaxed);
          cx.spawn(|this: gpui::WeakEntity<FolderSelectPage>, cx: &mut AsyncApp| {
            let mut owned_cx = cx.clone();
            async move {
              let folder = AsyncFileDialog::new().pick_folder().await;
              DIALOG_OPEN.store(false, Ordering::Relaxed);
              if let Some(f) = folder {
                let path = f.path().to_path_buf();
                owned_cx.update(|cx| cx.set_global(SelectedFolder(path)));
                let _ = this.update(&mut owned_cx, |_, cx| cx.notify());
              }
            }
          })
          .detach();
        })),
    );
    let path_label =
      div().text_color(cx.theme().muted_foreground).max_w(px(500.0)).truncate().child(path_display);

    let cache_section = div()
      .flex()
      .flex_col()
      .gap_1()
      .items_center()
      .child(div().text_sm().child("Cover cache size:"))
      .child(
        h_flex()
          .gap_2()
          .items_center()
          .child(NumberInput::new(self.cache_input.as_ref().unwrap()).small().w(px(120.0)))
          .child(
            div()
              .text_sm()
              .text_color(gray(500).opacity(0.6))
              .child(format!("/ {}", CACHE_SIZE_MAX)),
          ),
      );

    div()
      .flex()
      .flex_col()
      .gap_4()
      .size_full()
      .items_center()
      .justify_center()
      .child(header)
      .child(picker)
      .child(path_label)
      .child(cache_section)
  }
}
