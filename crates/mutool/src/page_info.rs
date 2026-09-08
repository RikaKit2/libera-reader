use crate::constants::{
  ARG_FORMAT, ARG_OUTPUT, ARG_QUIET, CMD_DRAW, CMD_PAGES, DEFAULT_A4_HEIGHT, DEFAULT_A4_WIDTH,
  FORMAT_TEXT, STDOUT_TARGET,
};
use crate::mutool_error::MuToolError;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::{Command, Stdio};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct PageDimensions {
  pub width: f32,
  pub height: f32,
}

impl Default for PageDimensions {
  fn default() -> Self {
    Self { width: DEFAULT_A4_WIDTH, height: DEFAULT_A4_HEIGHT }
  }
}

impl PageDimensions {
  pub const fn new(width: f32, height: f32) -> Self {
    Self { width, height }
  }

  pub fn aspect_ratio(&self) -> f32 {
    if self.height > 0.0 { self.width / self.height } else { 1.0 }
  }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct DocumentPageInfo {
  pub total_pages: usize,
  pub pages: Vec<PageDimensions>,
}

/// Retrieve the total page count and dimensions for each page in a document.
pub fn get_document_page_info(path_to_book: &Path) -> Result<DocumentPageInfo, MuToolError> {
  if !path_to_book.is_file() {
    return Err(MuToolError::IoError);
  }

  let output = Command::new("mutool")
    .arg(CMD_PAGES)
    .arg(path_to_book)
    .stdout(Stdio::piped())
    .stderr(Stdio::null())
    .output()
    .map_err(|_| MuToolError::MutoolNotFound)?;

  if !output.status.success() {
    // If mutool pages fails (e.g. for some EPUB/CBZ formats), fallback to probe
    return fallback_probe_page_info(path_to_book);
  }

  let stdout_str = String::from_utf8_lossy(&output.stdout);
  let page_info = parse_pages_output(&stdout_str);

  if page_info.total_pages == 0 {
    return fallback_probe_page_info(path_to_book);
  }

  Ok(page_info)
}

/// Quick helper to get only the total page count of a document.
pub fn get_page_count(path_to_book: &Path) -> Result<usize, MuToolError> {
  let info = get_document_page_info(path_to_book)?;
  Ok(info.total_pages)
}

/// Quick helper to get the dimension of a specific page (1-indexed).
pub fn get_page_size(path_to_book: &Path, page: usize) -> Result<PageDimensions, MuToolError> {
  let info = get_document_page_info(path_to_book)?;
  if page == 0 || page > info.total_pages {
    return Ok(PageDimensions::default());
  }
  Ok(info.pages.get(page - 1).copied().unwrap_or_default())
}

/// Parse the XML-like output of `mutool pages <file>`.
pub fn parse_pages_output(output: &str) -> DocumentPageInfo {
  let mut pages = Vec::new();
  let mut current_page_dims: Option<PageDimensions> = None;

  for line in output.lines() {
    let trimmed = line.trim();
    if trimmed.starts_with("<page ") {
      if let Some(dims) = current_page_dims.take() {
        pages.push(dims);
      }
      current_page_dims = Some(PageDimensions::default());
    } else if (trimmed.starts_with("<MediaBox ") || trimmed.starts_with("<CropBox "))
      && current_page_dims.is_some()
    {
      if let Some(dims) = parse_box_attributes(trimmed) {
        current_page_dims = Some(dims);
      }
    } else if trimmed.starts_with("</page>")
      && let Some(dims) = current_page_dims.take()
    {
      pages.push(dims);
    }
  }

  if let Some(dims) = current_page_dims.take() {
    pages.push(dims);
  }

  let total_pages = pages.len();
  DocumentPageInfo { total_pages, pages }
}

/// Extracts width and height from attributes like `l="0" b="0" r="595.28" t="841.89"`.
fn parse_box_attributes(line: &str) -> Option<PageDimensions> {
  let l = extract_attr_f32(line, "l=")?;
  let b = extract_attr_f32(line, "b=")?;
  let r = extract_attr_f32(line, "r=")?;
  let t = extract_attr_f32(line, "t=")?;

  let width = (r - l).abs();
  let height = (t - b).abs();

  if width > 0.0 && height > 0.0 { Some(PageDimensions::new(width, height)) } else { None }
}

fn extract_attr_f32(line: &str, attr_name: &str) -> Option<f32> {
  let start_pos = line.find(attr_name)?;
  let after_key = &line[start_pos + attr_name.len()..];
  let quote_char = after_key.chars().next()?;
  if quote_char != '"' && quote_char != '\'' {
    return None;
  }
  let value_str = &after_key[1..];
  let end_pos = value_str.find(quote_char)?;
  value_str[..end_pos].parse::<f32>().ok()
}

/// Fallback for formats where `mutool pages` doesn't output XML (e.g. text/epub),
/// using `mutool draw -F text -o - file 1-N` to count pages.
fn fallback_probe_page_info(path_to_book: &Path) -> Result<DocumentPageInfo, MuToolError> {
  let output = Command::new("mutool")
    .arg(CMD_DRAW)
    .arg(ARG_QUIET)
    .arg(ARG_FORMAT)
    .arg(FORMAT_TEXT)
    .arg(ARG_OUTPUT)
    .arg(STDOUT_TARGET)
    .arg(path_to_book)
    .stdout(Stdio::piped())
    .stderr(Stdio::null())
    .output()
    .map_err(|_| MuToolError::MutoolNotFound)?;

  if !output.status.success() {
    return Err(MuToolError::OtherErr);
  }

  // Count occurrences of page boundary or at least 1 page
  let stdout_str = String::from_utf8_lossy(&output.stdout);
  let count = stdout_str.matches("\x0C").count().max(1);

  let pages = vec![PageDimensions::default(); count];
  Ok(DocumentPageInfo { total_pages: count, pages })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_parse_pages_xml() {
    let xml = r#"
/path/to/doc.pdf:
<page pagenum="1">
<MediaBox l="0" b="0" r="595.28" t="841.89" />
</page>
<page pagenum="2">
<MediaBox l="0" b="0" r="800" t="600" />
</page>
"#;
    let info = parse_pages_output(xml);
    assert_eq!(info.total_pages, 2);
    assert_eq!(info.pages[0].width, 595.28);
    assert_eq!(info.pages[0].height, 841.89);
    assert_eq!(info.pages[1].width, 800.0);
    assert_eq!(info.pages[1].height, 600.0);
  }
}
