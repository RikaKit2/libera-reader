use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Default)]
pub struct Thumbnail {}

impl Thumbnail {
  pub fn new() -> Self {
    Thumbnail {}
  }
}
