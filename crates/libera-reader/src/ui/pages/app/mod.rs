use crate::router::Route;
use dioxus::prelude::*;

mod bookmarks;
mod favorite;
mod header;
mod history;
mod library;
mod settings;
mod stats;
mod side_bar;
pub(crate) use bookmarks::Bookmarks;
pub(crate) use favorite::Favorite;
pub(crate) use history::History;
pub(crate) use library::Library;
pub(crate) use settings::Settings;
pub(crate) use stats::Stats;
use side_bar::SideBar;

#[component]
pub fn App() -> Element {
    rsx! {
        div { class: "flex bg-base-100 h-screen text-[#9FB9D0]",
            SideBar {}
            div {
                class: "flex flex-col ml-[48px] bg-[#121C22]",
                width: "calc(100% - 48px)",
                Outlet::<Route> {}
            }
        }
    }
}
