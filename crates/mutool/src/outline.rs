use crate::constants::{CMD_SHOW, SHOW_TARGET_OUTLINE};
use crate::mutool_error::MuToolError;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::{Command, Stdio};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct OutlineNode {
  pub title: String,
  pub page: usize,
  pub depth: usize,
  pub uri: Option<String>,
  pub children: Vec<OutlineNode>,
}

impl OutlineNode {
  pub fn new(title: String, page: usize, depth: usize, uri: Option<String>) -> Self {
    Self { title, page, depth, uri, children: Vec::new() }
  }
}

/// Retrieve the hierarchical table of contents (outline) of a document.
pub fn get_document_outline(path_to_book: &Path) -> Result<Vec<OutlineNode>, MuToolError> {
  if !path_to_book.is_file() {
    return Err(MuToolError::IoError);
  }

  let output = Command::new("mutool")
    .arg(CMD_SHOW)
    .arg(path_to_book)
    .arg(SHOW_TARGET_OUTLINE)
    .stdout(Stdio::piped())
    .stderr(Stdio::null())
    .output()
    .map_err(|_| MuToolError::MutoolNotFound)?;

  if !output.status.success() {
    return Ok(Vec::new());
  }

  let stdout_str = String::from_utf8_lossy(&output.stdout);
  Ok(parse_outline_output(&stdout_str))
}

/// Parse lines from `mutool show file.pdf outline`.
/// Typical lines:
/// `+    "Chapter 1"    #page=1`
/// `|    +    "Section 1.1"    #page=2`
/// `|    \    "Section 1.2"    #page=3`
/// `\    "Chapter 2"    #page=4`
pub fn parse_outline_output(output: &str) -> Vec<OutlineNode> {
  let mut flat_items: Vec<OutlineNode> = Vec::new();

  for line in output.lines() {
    let trimmed_end = line.trim_end();
    if trimmed_end.is_empty() {
      continue;
    }

    let depth = calculate_line_depth(line);
    if let Some((title, page, uri)) = parse_outline_line(trimmed_end) {
      flat_items.push(OutlineNode::new(title, page, depth, uri));
    }
  }

  build_outline_tree(flat_items)
}

fn calculate_line_depth(line: &str) -> usize {
  let mut depth = 0;
  for ch in line.chars() {
    match ch {
      '|' | '\t' => depth += 1,
      ' ' => {}
      '+' | '\\' | '-' => break,
      _ => break,
    }
  }
  depth
}

fn parse_outline_line(line: &str) -> Option<(String, usize, Option<String>)> {
  // Find title inside quotes or tab-separated
  let title = if let Some(first_quote) = line.find('"') {
    let after_first = &line[first_quote + 1..];
    let second_quote = after_first.find('"')?;
    after_first[..second_quote].to_string()
  } else {
    // Fallback if not quoted
    let marker_pos = line.find(['+', '\\', '-'])?;
    let content = &line[marker_pos + 1..].trim();
    let end_pos = content.find('#').unwrap_or(content.len());
    content[..end_pos].trim().to_string()
  };

  let mut page = 1;
  let mut uri = None;

  if let Some(hash_pos) = line.find('#') {
    let target = &line[hash_pos + 1..];
    if let Some(stripped) = target.strip_prefix("page=") {
      page = stripped
        .split(|c: char| !c.is_ascii_digit())
        .next()
        .and_then(|p| p.parse::<usize>().ok())
        .unwrap_or(1);
    } else if let Some(digit_str) = target.split(|c: char| !c.is_ascii_digit()).next()
      && let Ok(p) = digit_str.parse::<usize>()
    {
      page = p;
    }
    uri = Some(target.to_string());
  } else if let Some(uri_start) = line.find("http://").or_else(|| line.find("https://")) {
    uri = Some(line[uri_start..].trim().to_string());
  }

  Some((title, page, uri))
}

fn build_outline_tree(flat: Vec<OutlineNode>) -> Vec<OutlineNode> {
  let mut root: Vec<OutlineNode> = Vec::new();
  let mut stack: Vec<OutlineNode> = Vec::new();

  for item in flat {
    while let Some(top) = stack.last() {
      if top.depth >= item.depth {
        let popped = stack.pop().unwrap();
        if let Some(parent) = stack.last_mut() {
          parent.children.push(popped);
        } else {
          root.push(popped);
        }
      } else {
        break;
      }
    }
    stack.push(item);
  }

  while let Some(popped) = stack.pop() {
    if let Some(parent) = stack.last_mut() {
      parent.children.push(popped);
    } else {
      root.push(popped);
    }
  }

  root
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_parse_outline_sample() {
    let sample = r#"
+	"Chapter 1: Getting Started"	#page=1
|	+	"1.1 Installation"	#page=5
|	\	"1.2 First Steps"	#page=12
\	"Chapter 2: Advanced Topics"	#page=25
"#;
    let tree = parse_outline_output(sample);
    assert_eq!(tree.len(), 2);
    assert_eq!(tree[0].title, "Chapter 1: Getting Started");
    assert_eq!(tree[0].page, 1);
    assert_eq!(tree[0].children.len(), 2);
    assert_eq!(tree[0].children[0].title, "1.1 Installation");
    assert_eq!(tree[0].children[0].page, 5);
    assert_eq!(tree[1].title, "Chapter 2: Advanced Topics");
    assert_eq!(tree[1].page, 25);
  }
}
