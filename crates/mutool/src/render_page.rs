use crate::constants::{
  ARG_FORMAT, ARG_OUTPUT, ARG_QUIET, ARG_RESOLUTION, CMD_DRAW, FORMAT_PNG, STDOUT_TARGET,
};
use crate::mutool_error::MuToolError;
use std::path::Path;
use std::process::{Command, Stdio};

/// Render a single document page (1-indexed) directly to a PNG file on disk.
pub fn render_page_to_png(
  path_to_book: &Path, page: usize, dpi: u32, output_path: &Path,
) -> Result<(), MuToolError> {
  if !path_to_book.is_file() {
    return Err(MuToolError::IoError);
  }

  let page_str = page.to_string();
  let dpi_str = dpi.to_string();

  let mut child = Command::new("mutool")
    .arg(CMD_DRAW)
    .arg(ARG_QUIET)
    .arg(ARG_RESOLUTION)
    .arg(&dpi_str)
    .arg(ARG_FORMAT)
    .arg(FORMAT_PNG)
    .arg(ARG_OUTPUT)
    .arg(output_path)
    .arg(path_to_book)
    .arg(&page_str)
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .spawn()
    .map_err(|_| MuToolError::MutoolNotFound)?;

  let status = child.wait().map_err(|_| MuToolError::IoError)?;
  MuToolError::from_process_exit_status(status)
}

/// Render a single document page (1-indexed) into memory as encoded PNG bytes.
pub fn render_page_to_png_bytes(
  path_to_book: &Path, page: usize, dpi: u32,
) -> Result<Vec<u8>, MuToolError> {
  if !path_to_book.is_file() {
    return Err(MuToolError::IoError);
  }

  let page_str = page.to_string();
  let dpi_str = dpi.to_string();

  let output = Command::new("mutool")
    .arg(CMD_DRAW)
    .arg(ARG_QUIET)
    .arg(ARG_RESOLUTION)
    .arg(&dpi_str)
    .arg(ARG_FORMAT)
    .arg(FORMAT_PNG)
    .arg(ARG_OUTPUT)
    .arg(STDOUT_TARGET)
    .arg(path_to_book)
    .arg(&page_str)
    .stdout(Stdio::piped())
    .stderr(Stdio::null())
    .output()
    .map_err(|_| MuToolError::MutoolNotFound)?;

  if !output.status.success() {
    return Err(MuToolError::OtherErr);
  }

  Ok(output.stdout)
}
