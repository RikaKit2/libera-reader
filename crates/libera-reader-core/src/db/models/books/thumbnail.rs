use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct Thumbnail {}

impl Thumbnail {
  pub fn new() -> Self {
    Thumbnail {}
  }
}
impl Default for Thumbnail {
  fn default() -> Self {
    Thumbnail {}
  }
}
