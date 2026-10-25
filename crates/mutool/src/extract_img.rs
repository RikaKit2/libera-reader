use crate::mutool_command;
use crate::mutool_error::MuToolError;
use image::ImageFormat;
use image::ImageReader;
use image::codecs::jpeg::JpegEncoder;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::process::Stdio;

pub async fn extract_img(
  path_to_book: &PathBuf, resolution: u32, path_to_thumbnail: &PathBuf,
) -> Result<(), MuToolError> {
  if path_to_thumbnail.exists()
    && std::fs::metadata(path_to_thumbnail).map(|m| m.len() > 0).unwrap_or(false)
  {
    return Ok(());
  }

  let parent = path_to_thumbnail.parent().ok_or(MuToolError::IoError)?.to_path_buf();
  std::fs::create_dir_all(&parent).map_err(|_| MuToolError::IoError)?;

  let ext = path_to_thumbnail.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();

  if ext == "webp" || ext == "jpg" || ext == "jpeg" {
    let temp_file =
      tempfile::Builder::new().suffix(".png").tempfile().map_err(|_| MuToolError::IoError)?;
    let temp_path = temp_file.path().to_path_buf();

    extract_img_with_format(path_to_book, resolution, &temp_path, "png").await?;

    let out_path = path_to_thumbnail.clone();
    let is_webp = ext == "webp";

    tokio::task::spawn_blocking(move || -> Result<(), MuToolError> {
      let reader = BufReader::new(File::open(&temp_path).map_err(|_| MuToolError::IoError)?);
      let img = ImageReader::new(reader)
        .with_guessed_format()
        .map_err(|_| MuToolError::OtherErr)?
        .decode()
        .map_err(|_| MuToolError::OtherErr)?;

      let mut tmp_out = tempfile::Builder::new()
        .prefix(".tmp_cover_")
        .tempfile_in(&parent)
        .map_err(|_| MuToolError::IoError)?;

      if is_webp {
        let rgb = img.to_rgb8();
        let encoder = webp::Encoder::from_rgb(rgb.as_raw(), rgb.width(), rgb.height());
        let webp_memory = encoder.encode(82.0);
        std::io::Write::write_all(&mut tmp_out, &webp_memory).map_err(|_| MuToolError::IoError)?;
        if tmp_out.persist(&out_path).is_err() {
          std::fs::write(&out_path, &*webp_memory).map_err(|_| MuToolError::IoError)?;
        }
      } else {
        let mut writer = std::io::BufWriter::new(&mut tmp_out);
        let encoder = JpegEncoder::new_with_quality(&mut writer, 85);
        img.write_with_encoder(encoder).map_err(|_| MuToolError::OtherErr)?;
        std::io::Write::flush(&mut writer).map_err(|_| MuToolError::IoError)?;
        drop(writer);
        if tmp_out.persist(&out_path).is_err() {
          let mut fallback_file = File::create(&out_path).map_err(|_| MuToolError::IoError)?;
          let mut fallback_writer = std::io::BufWriter::new(&mut fallback_file);
          let encoder = JpegEncoder::new_with_quality(&mut fallback_writer, 85);
          img.write_with_encoder(encoder).map_err(|_| MuToolError::OtherErr)?;
        }
      }

      Ok(())
    })
    .await
    .map_err(|_| MuToolError::OtherErr)??;

    Ok(())
  } else {
    extract_img_with_format(path_to_book, resolution, path_to_thumbnail, "png").await
  }
}

async fn extract_img_with_format(
  path_to_book: &PathBuf, resolution: u32, path_to_output: &PathBuf, format: &str,
) -> Result<(), MuToolError> {
  let mut child = mutool_command()
    .arg("draw")
    .arg("-q")
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
    .map_err(|_| MuToolError::MutoolNotFound)?;
  let status = child.wait().await.map_err(|_| MuToolError::IoError)?;
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

#[cfg(test)]
mod tests {
  use super::*;
  use tempfile::tempdir;

  #[tokio::test]
  async fn test_extract_jpeg_compression() {
    let tmp = tempdir().unwrap();
    let pdf_path = tmp.path().join("test.pdf");
    let cover_jpg = tmp.path().join("cover.jpg");

    crate::create_empty_book(&pdf_path, &pdf_path).await.unwrap();

    let res = extract_img(&pdf_path, 20, &cover_jpg).await;
    assert!(res.is_ok());
    assert!(cover_jpg.exists());
    let meta = std::fs::metadata(&cover_jpg).unwrap();
    assert!(meta.len() > 0);

    let reader = image::ImageReader::open(&cover_jpg).unwrap().with_guessed_format().unwrap();
    assert_eq!(reader.format(), Some(image::ImageFormat::Jpeg));
  }

  #[tokio::test]
  async fn test_extract_webp_compression() {
    let tmp = tempdir().unwrap();
    let pdf_path = tmp.path().join("test_webp.pdf");
    let cover_webp = tmp.path().join("cover.webp");

    crate::create_empty_book(&pdf_path, &pdf_path).await.unwrap();

    let res = extract_img(&pdf_path, 20, &cover_webp).await;
    assert!(res.is_ok());
    assert!(cover_webp.exists());
    let meta = std::fs::metadata(&cover_webp).unwrap();
    assert!(meta.len() > 0);

    let reader = image::ImageReader::open(&cover_webp).unwrap().with_guessed_format().unwrap();
    assert_eq!(reader.format(), Some(image::ImageFormat::WebP));
  }
}
