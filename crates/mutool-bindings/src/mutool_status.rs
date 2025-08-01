use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MUToolResult {
  Success,
  SIGSEGV,
  OtherErr,
}
