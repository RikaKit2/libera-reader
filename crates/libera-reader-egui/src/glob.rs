use crate::router::{BaseRoute, RootRoute, Router};
use once_cell::sync::Lazy;
use std::sync::RwLock;


pub(crate) const ROUTER: Lazy<RwLock<Router>> = Lazy::new(||
  RwLock::new(Router::new(RootRoute::Base(BaseRoute::Library)))
);
