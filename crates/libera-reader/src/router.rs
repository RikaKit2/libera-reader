use crate::ui::pages::app::{App, Bookmarks, Favorite, History, Library, Settings, Stats};
use crate::ui::pages::book_viewer::BookViewer;
use crate::ui::pages::setup::Setup;
use crate::ui::{PageNotFound, Root};
use dioxus::prelude::*;

#[derive(Routable, Clone, Debug, PartialEq)]
#[rustfmt::skip]
pub(crate) enum Route {
    #[layout(Root)]
      #[route("/")]
      Setup {},

      #[route("/book_viewer")]
      BookViewer {},

      #[layout(App)]
      
        #[route("/app/library")]
        Library {},

        #[route("/app/history")]
        History {},

        #[route("/app/favorite")]
        Favorite {},

        #[route("/app/bookmarks")]
        Bookmarks {},

        #[route("/app/stats")]
        Stats {},

        #[route("/app/settings")]
        Settings {},

      #[end_layout]

    #[end_layout]

    #[route("/:..route")]
    PageNotFound {
        route: Vec<String>,
    },
}
