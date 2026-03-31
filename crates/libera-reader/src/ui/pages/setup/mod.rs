use gpui::{App, AppContext, Context, Div, Entity, IntoElement, ParentElement, Render, Styled, Window, div};
use gpui_component::{
  Disableable,
  button::{Button, ButtonVariants},
};
use libera_reader_core::{
  ctx::GlobalCTX,
  db::models::{RootRoute, Route, settings::route::SetupRoute},
};
use rust_i18n::t;

use crate::ui::pages::setup::{
  appearance::Appearance, finish::Finish, library::Library, sync::SyncPage, tts::TTSPage, welcome::Welcome,
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
    Button::new("pages.setup.next_btn").primary().label(t!("pages.setup.next_btn")).disabled(disabled).on_click(
      |_event, _window, cx| {
        cx.ctx_mut().settings.to_next_setup_route();
      },
    ),
  )
}
fn back_btn() -> Div {
  div().child(Button::new("pages.setup.back_btn").label(t!("pages.setup.back_btn")).on_click(
    move |_event, _window, cx| {
      cx.ctx_mut().settings.to_previous_setup_route();
    },
  ))
}

fn finish_btn() -> Div {
  div().child(Button::new("pages.setup.finish_btn").primary().label(t!("pages.setup.finish_btn")).on_click(
    move |_event, _window, cx| {
      cx.ctx_mut().settings.set_setup_status(true).unwrap();
      cx.ctx_mut().settings.set_route(RootRoute::Main(Route::Library)).unwrap();
      cx.ctx_mut().db.compact().unwrap();
    },
  ))
}

impl Render for SetupPage {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    match cx.ctx().get_curr_route() {
      RootRoute::Main(_route) => div(),
      RootRoute::BookViewer => div(),
      RootRoute::Setup(setup_route) => match setup_route {
        SetupRoute::Welcome => div().w_full().h_full().child(self.welcome_page.clone()),
        SetupRoute::Appearance => div().w_full().h_full().child(self.appearance_page.clone()),
        SetupRoute::Library => div().w_full().h_full().child(self.library_page.clone()),
        SetupRoute::Sync => div().w_full().h_full().child(self.sync_page.clone()),
        SetupRoute::TTS => div().w_full().h_full().child(self.tts_page.clone()),
        SetupRoute::Finish => div().w_full().h_full().child(self.finish_page.clone()),
      },
    }
  }
}
