use mutool_bindings::mutool_status::MuToolError;
use serde::{Deserialize, Serialize};

use crate::db::models::Thumbnail;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MutoolData {
  pub mutool_err: Option<MuToolError>,
  pub title: Option<String>,
  pub author: Option<String>,
  pub page_count: Option<usize>,
  pub thumbnail: Thumbnail,
}

impl MutoolData {
  pub fn new(mutool_err: Option<MuToolError>, title: Option<String>, author: Option<String>, page_count: Option<usize>, thumbnail: Thumbnail) -> Self {
    MutoolData { mutool_err, title, author, page_count, thumbnail }
  }
  pub fn new_if_size_eq_zero() -> Self {
    MutoolData::new(Some(MuToolError::FileIsEmpty), None, None, None, Thumbnail::default())
  }
  pub fn is_cached(&self) -> bool {
    self.thumbnail.cached
  }
}
impl Default for MutoolData {
  fn default() -> Self {
    MutoolData { mutool_err: None, title: None, author: None, page_count: None, thumbnail: Thumbnail::default() }
  }
}
