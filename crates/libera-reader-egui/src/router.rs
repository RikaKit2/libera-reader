#[derive(Debug)]
pub enum RootRoute {
  Base(BaseRoute),
  BookViewer,
  Setup,
}

#[derive(PartialEq, Debug, Clone)]
pub enum BaseRoute {
  Library,
  FileManager,
  History,
  Favorite,
  BookMarks,
  Stats,
  Settings,
}

pub(crate) struct Router {
  inn: RootRoute,
}
impl Router {
  pub(crate) fn new(route: RootRoute) -> Self { Self { inn: route } }
  pub(crate) fn change_route(&mut self, route: RootRoute) {
    self.inn = route;
  }
  pub(crate) fn compare_with_root_route(&self, other: &BaseRoute) -> bool {
    match &self.inn {
      RootRoute::Base(base_route) => { other.eq(base_route) }
      RootRoute::BookViewer => { false }
      RootRoute::Setup => { false }
    }
  }
}
