use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Thumbnail {
  pub cached: bool,
}

impl Thumbnail {
  pub fn new(cached: bool) -> Self {
    Thumbnail { cached }
  }
}
impl Default for Thumbnail {
  fn default() -> Self {
    Thumbnail { cached: false }
  }
}
