use crate::mutool_error::MuToolError;
use image::ImageFormat;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;

pub async fn extract_img(
  path_to_book: &PathBuf, resolution: u32, path_to_thumbnail: &PathBuf,
) -> Result<(), MuToolError> {
  match path_to_thumbnail.exists() {
    true => Ok(()),
    false => {
      match extract_img_with_format(path_to_book, resolution, path_to_thumbnail, "png").await {
        Ok(_) => Ok(()),
        Err(err) => Err(err),
      }
    }
  }
}

async fn extract_img_with_format(
  path_to_book: &PathBuf, resolution: u32, path_to_output: &PathBuf, format: &str,
) -> Result<(), MuToolError> {
  let mut child = Command::new("mutool")
    .arg("draw")
    .arg("-r")
    .arg(resolution.to_string())
    .arg("-F")
    .arg(format)
    .arg("-o")
    .arg(path_to_output)
    .arg(path_to_book)
    .arg("1")
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .spawn()
    .unwrap();
  let status = child.wait().await.unwrap();
  MuToolError::from_process_exit_status(status)
}

pub async fn extract_to_bytes(
  path_to_book: &PathBuf, resolution: u32,
) -> Result<Vec<u8>, MuToolError> {
  // Mutool doesn't support JPEG output — extract as PNG then convert in memory
  let temp_file = tempfile::NamedTempFile::new().map_err(|_| MuToolError::IoError)?;
  let temp_path = temp_file.path().to_path_buf();

  extract_img_with_format(path_to_book, resolution, &temp_path, "png").await?;
  let data = imp_to_jpeg(&temp_path).map_err(|_| MuToolError::OtherErr)?;
  Ok(data)
}

pub async fn save_thumbnail(path_to_thumbnail: &PathBuf) {
  // PNG → JPEG conversion on disk (kept for compatibility; callers expect this)
  let data = imp_to_jpeg(path_to_thumbnail).unwrap();
  tokio::fs::remove_file(path_to_thumbnail).await.unwrap();
  tokio::fs::write(path_to_thumbnail.with_extension("jpeg"), data).await.unwrap();
}

fn imp_to_jpeg(path_to_thumbnail: &PathBuf) -> Result<Vec<u8>, anyhow::Error> {
  let reader = BufReader::new(File::open(path_to_thumbnail)?);
  let image = image::load(reader, ImageFormat::Png)?;
  let mut output = std::io::Cursor::new(Vec::new());
  image.write_to(&mut output, ImageFormat::Jpeg)?;
  Ok(output.into_inner())
}
