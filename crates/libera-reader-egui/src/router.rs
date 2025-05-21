#[derive(Debug)]
pub(crate) enum RootRoute {
  Base(Route),
  BookViewer,
  Setup,
}

#[derive(PartialEq, Debug, Clone)]
pub(crate) enum Route {
  Library,
  FileManager,
  History,
  Favorite,
  BookMarks,
  Stats,
  Settings,
}

#[derive(Debug)]
pub(crate) struct Router {
  pub(crate) inn: RootRoute,
}
impl Router {
  pub(crate) fn new(route: RootRoute) -> Self { Self { inn: route } }
  pub(crate) fn set_route(&mut self, route: RootRoute) {
    self.inn = route;
  }
  pub(crate) fn compare_with_root_route(&self, other: &Route) -> bool {
    match &self.inn {
      RootRoute::Base(base_route) => { other.eq(base_route) }
      RootRoute::BookViewer => { false }
      RootRoute::Setup => { false }
    }
  }
}
