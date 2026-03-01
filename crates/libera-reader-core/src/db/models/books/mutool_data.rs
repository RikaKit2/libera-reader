use mutool::mutool_status::MuToolError;
use serde::{Deserialize, Serialize};

use crate::db::models::Thumbnail;

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct MutoolData {
  pub mutool_err: Option<MuToolError>,
  pub title: Option<String>,
  pub author: Option<String>,
  pub page_count: Option<usize>,
  pub thumbnail: Option<Thumbnail>,
}

impl MutoolData {
  pub fn new(
    mutool_err: Option<MuToolError>, title: Option<String>, author: Option<String>, page_count: Option<usize>,
    thumbnail: Option<Thumbnail>,
  ) -> Self {
    MutoolData { mutool_err, title, author, page_count, thumbnail }
  }
  pub fn is_cached(&self) -> bool {
    self.thumbnail.is_some()
  }
}
impl Default for MutoolData {
  fn default() -> Self {
    MutoolData::new(None, None, None, None, None)
  }
}
