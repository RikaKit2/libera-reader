pub enum Route {
  Main(MainRoute),
  BookViewer,
  Setup,
}

#[derive(PartialEq, Debug)]
pub enum MainRoute {
  Library,
  FileManager,
  History,
  Favorite,
  BookMarks,
  Stats,
  Settings,
}
