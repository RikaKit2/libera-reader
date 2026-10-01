#![forbid(unsafe_code)]

pub mod command;
pub mod constants;
pub mod create_book;
pub mod download_mutool;
pub mod extract_img;
pub mod links;
pub mod mutool_error;
pub mod outline;
pub mod page_info;
pub mod render_page;
pub mod search;
pub mod stext;
pub use command::{get_mutool_bin_path, mutool_command, mutool_std_command, set_mutool_path};
pub use constants::*;
pub use create_book::create_empty_book;
pub use download_mutool::{download_mutool, download_mutool_if_missing_blocking};
pub use extract_img::{extract_img, extract_to_bytes, save_thumbnail};
pub use links::{PageLink, get_page_links};
pub use mutool_error::MuToolError;
pub use outline::{OutlineNode, get_document_outline};
pub use page_info::{
  DocumentPageInfo, PageDimensions, get_document_page_info, get_page_count, get_page_size,
};
pub use render_page::{
  render_page_to_png, render_page_to_png_bytes, render_page_to_webp, render_page_to_webp_bytes,
};
pub use search::{DocumentSearchMatch, search_document_text};
pub use stext::{
  BBox, FontInfo, PageStructuredText, TextBlock, TextLine, get_page_structured_text,
};

use std::path::{Path, PathBuf};

pub fn get_path_to_mutool(path_to_storage_dir: &Path) -> PathBuf {
  if cfg!(windows) {
    path_to_storage_dir.join("mutool.exe")
  } else {
    path_to_storage_dir.join("mutool")
  }
}
