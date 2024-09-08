use crate::router::MainRoute;
use dioxus::prelude::*;

mod header;
mod side_bar;
mod content;

use content::bookmarks::Bookmarks;
use content::favorite::Favorite;
use content::history::History;
use content::library::Library;
use content::settings::Settings;
use content::stats::Stats;
use side_bar::SideBar;

#[component]
pub fn Main(route: Signal<MainRoute>) -> Element {
  rsx! {
    div { class: "flex bg-base-100 h-screen text-[#9FB9D0]",
        SideBar { route }
        div {
            class: "flex flex-col ml-[48px] bg-[#121C22]",
            width: "calc(100% - 48px)",
            if route() == MainRoute::Library {
                Library {}
            } else if route() == MainRoute::History {
                History {}
            } else if route() == MainRoute::Favorite {
                Favorite {}
            } else if route() == MainRoute::Bookmarks {
                Bookmarks {}
            } else if route() == MainRoute::Stats {
                Stats {}
            } else if route() == MainRoute::Settings {
                Settings {}
            }
        }
    }
}
}
