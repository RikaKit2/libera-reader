pub enum Route {
  Main(MainRoute),
  BookViewer,
  Setup
}
#[derive(PartialEq)]
pub enum MainRoute {
  Library,
  FileManager,
  History,
  Favorite,
  BookMarks,
  State,
  Settings,
}
