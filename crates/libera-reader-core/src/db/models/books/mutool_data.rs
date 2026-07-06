use gpui::SharedString;
use mutool::mutool_error::MuToolError;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct MutoolData {
  pub mutool_err: Option<MuToolError>,
  pub title: Option<SharedString>,
  pub author: Option<SharedString>,
  pub page_count: Option<usize>,
}

impl MutoolData {
  pub fn new(
    mutool_err: Option<MuToolError>, title: Option<SharedString>, author: Option<SharedString>,
    page_count: Option<usize>,
  ) -> Self {
    MutoolData { mutool_err, title, author, page_count }
  }
}
impl Default for MutoolData {
  fn default() -> Self {
    MutoolData::new(None, None, None, None)
  }
}
