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
  pub(crate) inn: RootRoute,
}
impl Router {
  pub(crate) fn new(route: RootRoute) -> Self { Self { inn: route } }
  pub(crate) fn change_route(&mut self, route: RootRoute) {
    println!("{:?}", &route);
    self.inn = route;
  }
  pub(crate) fn compare_with_root_route(&self, other: &BaseRoute) -> bool {
    match &self.inn {
      RootRoute::Base(base_route) => { other.eq(base_route) }
      RootRoute::BookViewer => { false }
      RootRoute::Setup => { false }
    }
  }
  pub(crate) fn get_base_route(&self) -> Option<&BaseRoute> {
    match &self.inn {
      RootRoute::Base(res) => { Some(res) }
      RootRoute::BookViewer => { None }
      RootRoute::Setup => { None }
    }
  }
}
