use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialOrd, PartialEq, Copy, Debug)]
pub enum Route {
  Library,
  FileManager,
  History,
  Favorite,
  BookMarks,
  Stats,
  Settings,
}
#[derive(Serialize, Deserialize, Clone, PartialOrd, PartialEq, Copy, Debug)]
pub enum RootRoute {
  Main(Route),
  BookViewer,
  Setup,
}
