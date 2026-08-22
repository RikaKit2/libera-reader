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
pub enum SetupRoute {
  Welcome,
  Appearance,
  Library,
  Sync,
  TTS,
  Finish,
}

#[derive(Serialize, Deserialize, Clone, PartialOrd, PartialEq, Copy, Debug)]
pub enum RootRoute {
  Main(Route),
  BookViewer,
  Setup(SetupRoute),
}

impl SetupRoute {
  pub fn next(&self) -> Option<Self> {
    match self {
      Self::Welcome => Some(Self::Appearance),
      Self::Appearance => Some(Self::Library),
      Self::Library => Some(Self::Sync),
      Self::Sync => Some(Self::TTS),
      Self::TTS => Some(Self::Finish),
      Self::Finish => None,
    }
  }

  pub fn back(&self) -> Option<Self> {
    match self {
      Self::Welcome => None,
      Self::Appearance => Some(Self::Welcome),
      Self::Library => Some(Self::Appearance),
      Self::Sync => Some(Self::Library),
      Self::TTS => Some(Self::Sync),
      Self::Finish => Some(Self::TTS),
    }
  }
}
