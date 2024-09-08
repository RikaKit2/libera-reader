use crate::router::{BaseRoute, MainRoute};
use crate::ui::pages::book_viewer::BookViewer;
use crate::ui::pages::main::Main;
use crate::ui::pages::setup::Setup;
use dioxus::prelude::*;

#[component]
pub(crate) fn App() -> Element {
  let setup_is_ready = use_signal(|| true);
  let base_route = use_signal(|| {
    if setup_is_ready() {
      BaseRoute::Main
    } else {
      BaseRoute::Setup
    }
  });
  let main_route = use_signal(|| MainRoute::Library);

  if base_route() == BaseRoute::Setup {
    rsx! {
        Setup {}
    }
  } else if base_route() == BaseRoute::BookViewer {
    rsx! {
        BookViewer {}
    }
  } else if base_route() == BaseRoute::Main {
    rsx! {
        Main { route: main_route }
    }
  } else {
    rsx! {
        div { "page not found" }
    }
  }
}
