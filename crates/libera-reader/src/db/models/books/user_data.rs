use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Default)]
pub struct UserData {
  pub favorite: bool,
  #[serde(default)]
  pub last_opened: u64,
}
