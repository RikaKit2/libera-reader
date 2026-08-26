use crate::app_ext::AppExt;
use crate::db::models::{RootRoute, Route, settings::route::SetupRoute};
use crate::services::start_services;
use gpui::{
  App, AppContext, Context, Div, Entity, IntoElement, ParentElement, Render, Styled, Window, div,
};
use gpui_component::{
  Disableable,
  button::{Button, ButtonVariants},
};
use rust_i18n::t;

use crate::ui::pages::setup::{
  appearance::Appearance, finish::Finish, library::Library, sync::SyncPage, tts::TTSPage,
  welcome::Welcome,
};

mod appearance;
mod finish;
mod library;
mod sync;
mod tts;
mod welcome;

pub(crate) struct SetupPage {
  welcome_page: Entity<Welcome>,
  appearance_page: Entity<Appearance>,
  library_page: Entity<Library>,
  sync_page: Entity<SyncPage>,
  tts_page: Entity<TTSPage>,
  finish_page: Entity<Finish>,
}
impl SetupPage {
  pub(crate) fn new(window: &mut Window, cx: &mut App) -> Entity<Self> {
    cx.new(|cx| Self {
      welcome_page: Welcome::new(cx),
      appearance_page: Appearance::new(window, cx),
      library_page: Library::new(cx),
      sync_page: SyncPage::new(cx),
      tts_page: TTSPage::new(cx),
      finish_page: Finish::new(cx),
    })
  }
}

fn next_btn(disabled: bool) -> Div {
  div().child(
    Button::new("pages.setup.next_btn")
      .primary()
      .label(t!("pages.setup.next_btn"))
      .disabled(disabled)
      .on_click(|_event, _window, cx| {
        cx.settings_mut().to_next_setup_route();
      }),
  )
}
fn back_btn() -> Div {
  div().child(Button::new("pages.setup.back_btn").label(t!("pages.setup.back_btn")).on_click(
    move |_event, _window, cx| {
      cx.settings_mut().to_previous_setup_route();
    },
  ))
}

fn finish_btn() -> Div {
  div().child(
    Button::new("pages.setup.finish_btn").primary().label(t!("pages.setup.finish_btn")).on_click(
      move |_event, _window, cx| {
        cx.settings_mut().set_setup_status(true).unwrap();
        cx.settings_mut().set_route(RootRoute::Main(Route::Library)).unwrap();
        cx.db().compact().unwrap();
        start_services(cx);
      },
    ),
  )
}

impl Render for SetupPage {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    match cx.settings().read().route {
      RootRoute::Setup(setup_route) => {
        let page = match setup_route {
          SetupRoute::Welcome => self.welcome_page.clone().into_any_element(),
          SetupRoute::Appearance => self.appearance_page.clone().into_any_element(),
          SetupRoute::Library => self.library_page.clone().into_any_element(),
          SetupRoute::Sync => self.sync_page.clone().into_any_element(),
          SetupRoute::TTS => self.tts_page.clone().into_any_element(),
          SetupRoute::Finish => self.finish_page.clone().into_any_element(),
        };
        div().size_full().child(page)
      }
      _ => div(),
    }
  }
}
