use crate::mutool_error::MuToolError;
use crate::stext::BBox;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Stdio;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct DocumentSearchMatch {
  pub page: usize,
  pub bbox: BBox,
  #[serde(default)]
  pub snippet: String,
}

impl DocumentSearchMatch {
  pub fn new(page: usize, bbox: BBox, snippet: impl Into<String>) -> Self {
    Self { page, bbox, snippet: snippet.into() }
  }
}

const SEARCH_SCRIPT_CONTENT: &str = r#"
var doc = mupdf.Document.openDocument(mupdf.scriptArgs[0]);
var needle = mupdf.scriptArgs[1];
var targetPage = mupdf.scriptArgs[2] ? parseInt(mupdf.scriptArgs[2]) : null;

var startPage = targetPage !== null ? targetPage - 1 : 0;
var endPage = targetPage !== null ? targetPage : doc.countPages();

var allHits = [];
for (var p = startPage; p < endPage; p++) {
    var page = doc.loadPage(p);
    var hits = page.toStructuredText().search(needle);
    for (var i = 0; i < hits.length; i++) {
        var hit = hits[i];
        for (var j = 0; j < hit.length; j++) {
            var q = hit[j];
            var x0 = Math.min(q[0], q[2], q[4], q[6]);
            var y0 = Math.min(q[1], q[3], q[5], q[7]);
            var x1 = Math.max(q[0], q[2], q[4], q[6]);
            var y1 = Math.max(q[1], q[3], q[5], q[7]);
            allHits.push({
                page: p + 1,
                bbox: { x: x0, y: y0, w: x1 - x0, h: y1 - y0 }
            });
        }
    }
}
print(JSON.stringify(allHits));
"#;

fn ensure_search_script() -> Result<PathBuf, MuToolError> {
  let script_path = std::env::temp_dir().join("libera_mupdf_search_v1.js");
  if !script_path.is_file() {
    std::fs::write(&script_path, SEARCH_SCRIPT_CONTENT).map_err(|_| MuToolError::IoError)?;
  }
  Ok(script_path)
}

/// Perform a full-text search across all pages of a document using `mutool run`.
/// Returns exact character bounding boxes calculated by MuPDF's internal font engine.
pub fn search_document_text(
  path_to_book: &Path, query: &str,
) -> Result<Vec<DocumentSearchMatch>, MuToolError> {
  if !path_to_book.is_file() {
    return Err(MuToolError::IoError);
  }

  let query_trimmed = query.trim();
  if query_trimmed.is_empty() {
    return Ok(Vec::new());
  }

  let script_path = ensure_search_script()?;

  let output = crate::mutool_std_command()
    .arg("run")
    .arg(&script_path)
    .arg(path_to_book)
    .arg(query_trimmed)
    .stdout(Stdio::piped())
    .stderr(Stdio::null())
    .output()
    .map_err(|_| MuToolError::MutoolNotFound)?;

  if !output.status.success() {
    return Err(MuToolError::OtherErr);
  }

  let stdout_str = String::from_utf8_lossy(&output.stdout);
  parse_search_json_output(&stdout_str, query_trimmed)
}

/// Parse JSON array of search results emitted by the search script.
pub fn parse_search_json_output(
  raw_output: &str, fallback_snippet: &str,
) -> Result<Vec<DocumentSearchMatch>, MuToolError> {
  let json_start = match raw_output.find('[') {
    Some(idx) => idx,
    None => return Ok(Vec::new()),
  };

  let json_end = match raw_output.rfind(']') {
    Some(idx) => idx + 1,
    None => return Ok(Vec::new()),
  };

  let slice = &raw_output[json_start..json_end];

  #[derive(Deserialize)]
  struct RawHit {
    page: usize,
    bbox: BBox,
  }

  let raw_hits: Vec<RawHit> = serde_json::from_str(slice)
    .map_err(|e| MuToolError::ParseError(format!("Failed to parse search hits JSON: {}", e)))?;

  let matches = raw_hits
    .into_iter()
    .map(|hit| DocumentSearchMatch::new(hit.page, hit.bbox, fallback_snippet))
    .collect();

  Ok(matches)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_parse_search_json_output() {
    let json = r#"[{"bbox":{"h":15.7,"w":61.4,"x":72.0,"y":174.5},"page":2}]"#;
    let matches = parse_search_json_output(json, "test query").unwrap();

    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].page, 2);
    assert_eq!(matches[0].bbox.x, 72.0);
    assert_eq!(matches[0].bbox.y, 174.5);
    assert_eq!(matches[0].bbox.w, 61.4);
    assert_eq!(matches[0].bbox.h, 15.7);
    assert_eq!(matches[0].snippet, "test query");
  }

  #[test]
  fn test_parse_search_json_output_empty() {
    let json = "[]";
    let matches = parse_search_json_output(json, "query").unwrap();
    assert!(matches.is_empty());
  }

  #[test]
  fn test_parse_search_json_output_no_brackets() {
    let json = "No matches";
    let matches = parse_search_json_output(json, "query").unwrap();
    assert!(matches.is_empty());
  }
}
