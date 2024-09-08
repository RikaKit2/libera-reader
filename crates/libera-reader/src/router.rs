#[derive(Clone, Debug, PartialEq)]
pub(crate) enum MainRoute {
  Library,
  History,
  Favorite,
  Bookmarks,
  Stats,
  Settings,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum BaseRoute {
  Main,
  Setup,
  BookViewer,
}
