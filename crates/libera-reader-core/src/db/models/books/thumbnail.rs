use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct Thumbnail {
  pub data: Vec<u8>,
}

impl Thumbnail {
  pub fn new(data: Vec<u8>) -> Self {
    Thumbnail { data }
  }
}
