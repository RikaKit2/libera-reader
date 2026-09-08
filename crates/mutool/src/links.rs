use crate::constants::{CMD_SHOW, SHOW_TARGET_LINKS};
use crate::mutool_error::MuToolError;
use crate::stext::BBox;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::{Command, Stdio};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PageLink {
  pub bbox: BBox,
  pub uri: String,
  pub dest_page: Option<usize>,
}

impl PageLink {
  pub fn new(bbox: BBox, uri: String, dest_page: Option<usize>) -> Self {
    Self { bbox, uri, dest_page }
  }
}

/// Retrieve interactive hyperlinks for a specific page.
pub fn get_page_links(path_to_book: &Path, _page: usize) -> Result<Vec<PageLink>, MuToolError> {
  if !path_to_book.is_file() {
    return Err(MuToolError::IoError);
  }

  let output = Command::new("mutool")
    .arg(CMD_SHOW)
    .arg(path_to_book)
    .arg(SHOW_TARGET_LINKS)
    .stdout(Stdio::piped())
    .stderr(Stdio::null())
    .output()
    .map_err(|_| MuToolError::MutoolNotFound)?;

  if !output.status.success() {
    return Ok(Vec::new());
  }

  let stdout_str = String::from_utf8_lossy(&output.stdout);
  Ok(parse_links_output(&stdout_str))
}

pub fn parse_links_output(output: &str) -> Vec<PageLink> {
  let mut links = Vec::new();
  for line in output.lines() {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed == "null" {
      continue;
    }
    // Parse link line if format matches "[x y w h] -> uri/dest"
    if let Some(link) = parse_single_link_line(trimmed) {
      links.push(link);
    }
  }
  links
}

fn parse_single_link_line(line: &str) -> Option<PageLink> {
  // Format typically: [x0 y0 x1 y1] -> uri
  let open_bracket = line.find('[')?;
  let close_bracket = line.find(']')?;
  let coords_str = &line[open_bracket + 1..close_bracket];
  let parts: Vec<f32> =
    coords_str.split_whitespace().filter_map(|s| s.parse::<f32>().ok()).collect();

  if parts.len() < 4 {
    return None;
  }

  let x = parts[0];
  let y = parts[1];
  let w = (parts[2] - parts[0]).abs();
  let h = (parts[3] - parts[1]).abs();

  let target_part = line[close_bracket + 1..].trim();
  let uri = target_part.trim_start_matches("->").trim().to_string();

  let dest_page = if let Some(stripped) = uri.strip_prefix("#page=") {
    stripped.split(|c: char| !c.is_ascii_digit()).next().and_then(|p| p.parse::<usize>().ok())
  } else {
    None
  };

  Some(PageLink::new(BBox::new(x, y, w, h), uri, dest_page))
}
