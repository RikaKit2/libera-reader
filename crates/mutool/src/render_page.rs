use crate::constants::{
  ARG_FORMAT, ARG_OUTPUT, ARG_QUIET, ARG_RESOLUTION, CMD_DRAW, FORMAT_PNG, STDOUT_TARGET,
};
use crate::mutool_error::MuToolError;
use std::path::Path;
use std::process::Stdio;

/// Render a single document page (1-indexed) directly to a PNG file on disk.
pub fn render_page_to_png(
  path_to_book: &Path, page: usize, dpi: u32, output_path: &Path,
) -> Result<(), MuToolError> {
  if !path_to_book.is_file() {
    return Err(MuToolError::IoError);
  }

  let page_str = page.to_string();
  let dpi_str = dpi.to_string();

  let mut child = crate::mutool_std_command()
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

  let output = crate::mutool_std_command()
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

/// Render a single document page (1-indexed) directly to encoded WebP bytes.
pub fn render_page_to_webp_bytes(
  path_to_book: &Path, page: usize, dpi: u32, quality: f32,
) -> Result<Vec<u8>, MuToolError> {
  let png_bytes = render_page_to_png_bytes(path_to_book, page, dpi)?;
  let img = image::ImageReader::new(std::io::Cursor::new(png_bytes))
    .with_guessed_format()
    .map_err(|_| MuToolError::OtherErr)?
    .decode()
    .map_err(|_| MuToolError::OtherErr)?;

  let encoder = webp::Encoder::from_image(&img).map_err(|_| MuToolError::OtherErr)?;
  let webp_memory = encoder.encode(quality);
  Ok(webp_memory.to_vec())
}

/// Render a single document page (1-indexed) directly to a WebP file on disk.
pub fn render_page_to_webp(
  path_to_book: &Path, page: usize, dpi: u32, output_path: &Path, quality: f32,
) -> Result<(), MuToolError> {
  let webp_bytes = render_page_to_webp_bytes(path_to_book, page, dpi, quality)?;
  if let Some(parent) = output_path.parent() {
    let _ = std::fs::create_dir_all(parent);
  }
  std::fs::write(output_path, webp_bytes).map_err(|_| MuToolError::IoError)?;
  Ok(())
}
