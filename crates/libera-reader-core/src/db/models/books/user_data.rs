use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct UserData {
  pub favorite: bool,
  pub in_history: bool,
}

impl UserData {
  pub fn new(favorite: bool, in_history: bool) -> Self {
    UserData { favorite, in_history }
  }
}

impl Default for UserData {
  fn default() -> Self {
    UserData { favorite: false, in_history: false }
  }
}
