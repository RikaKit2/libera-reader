use crate::constants::{
  ARG_FORMAT, ARG_OUTPUT, ARG_QUIET, CMD_DRAW, FORMAT_STEXT_JSON, STDOUT_TARGET,
};
use crate::mutool_error::MuToolError;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::{Command, Stdio};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Default)]
pub struct BBox {
  pub x: f32,
  pub y: f32,
  pub w: f32,
  pub h: f32,
}

impl BBox {
  pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
    Self { x, y, w, h }
  }

  pub fn scaled(&self, factor: f32) -> Self {
    Self { x: self.x * factor, y: self.y * factor, w: self.w * factor, h: self.h * factor }
  }

  pub fn contains_point(&self, px: f32, py: f32) -> bool {
    px >= self.x && px <= (self.x + self.w) && py >= self.y && py <= (self.y + self.h)
  }

  pub fn intersects(&self, other: &Self) -> bool {
    self.x < (other.x + other.w)
      && (self.x + self.w) > other.x
      && self.y < (other.y + other.h)
      && (self.y + self.h) > other.y
  }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct FontInfo {
  pub name: Option<String>,
  pub family: Option<String>,
  pub weight: Option<String>,
  pub style: Option<String>,
  #[serde(default)]
  pub size: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct TextLine {
  pub bbox: BBox,
  #[serde(default)]
  pub font: Option<FontInfo>,
  #[serde(default)]
  pub text: String,
  #[serde(default)]
  pub x: f32,
  #[serde(default)]
  pub y: f32,
  #[serde(default)]
  pub wmode: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TextBlock {
  #[serde(rename = "type")]
  pub block_type: String,
  pub bbox: BBox,
  #[serde(default)]
  pub lines: Vec<TextLine>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct PageStructuredText {
  #[serde(default)]
  pub blocks: Vec<TextBlock>,
}

#[derive(Deserialize, Debug)]
struct MuToolSTextDoc {
  #[serde(default)]
  pub pages: Vec<PageStructuredText>,
}

/// Retrieve structured text (bounding boxes, font info, spans) for a specific page (1-indexed).
pub fn get_page_structured_text(
  path_to_book: &Path, page: usize,
) -> Result<PageStructuredText, MuToolError> {
  if !path_to_book.is_file() {
    return Err(MuToolError::IoError);
  }

  let page_arg = page.to_string();
  let output = Command::new("mutool")
    .arg(CMD_DRAW)
    .arg(ARG_QUIET)
    .arg(ARG_FORMAT)
    .arg(FORMAT_STEXT_JSON)
    .arg(ARG_OUTPUT)
    .arg(STDOUT_TARGET)
    .arg(path_to_book)
    .arg(&page_arg)
    .stdout(Stdio::piped())
    .stderr(Stdio::null())
    .output()
    .map_err(|_| MuToolError::MutoolNotFound)?;

  if !output.status.success() {
    return Err(MuToolError::OtherErr);
  }

  let stdout_str = String::from_utf8_lossy(&output.stdout);
  parse_stext_json(&stdout_str)
}

/// Parse raw stdout from `mutool draw -F stext.json` into `PageStructuredText`.
pub fn parse_stext_json(raw_output: &str) -> Result<PageStructuredText, MuToolError> {
  let json_start = raw_output.find('{').ok_or_else(|| {
    MuToolError::ParseError("no JSON object found in mutool stext output".to_string())
  })?;

  let json_str = &raw_output[json_start..];
  let doc: MuToolSTextDoc = serde_json::from_str(json_str)
    .map_err(|e| MuToolError::JsonError(format!("failed to parse stext JSON: {e}")))?;

  Ok(doc.pages.into_iter().next().unwrap_or_default())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_parse_stext_json() {
    let raw = r#"page /test.pdf 1{"file":"/test.pdf","pages":[{"blocks":[{"type":"text","bbox":{"x":10.0,"y":20.0,"w":100.0,"h":30.0},"lines":[{"wmode":0,"bbox":{"x":10.0,"y":20.0,"w":50.0,"h":15.0},"font":{"name":"Arial","family":"sans-serif","weight":"normal","style":"normal","size":12.0},"x":10.0,"y":32.0,"text":"Hello world"}]}]}]}"#;
    let stext = parse_stext_json(raw).unwrap();
    assert_eq!(stext.blocks.len(), 1);
    assert_eq!(stext.blocks[0].lines.len(), 1);
    assert_eq!(stext.blocks[0].lines[0].text, "Hello world");
    assert_eq!(stext.blocks[0].lines[0].bbox.w, 50.0);
  }
}
