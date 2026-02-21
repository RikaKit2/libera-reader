use crate::mutool_status::MuToolError;
use image::{GenericImageView, ImageFormat};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;
use zune_core::bit_depth::BitDepth;
use zune_core::colorspace::ColorSpace;
use zune_core::options::EncoderOptions;
use zune_jpegxl::{JxlEncodeErrors, JxlSimpleEncoder};

pub async fn extract_img(path_to_book: &PathBuf, resolution: u32, path_to_thumbnail: &PathBuf) -> Result<(), MuToolError> {
  match path_to_thumbnail.exists() {
    true => Ok(()),
    false => match extract_img_inn(path_to_book, resolution, path_to_thumbnail).await {
      Ok(_) => Ok(()),
      Err(err) => Err(err),
    },
  }
}
async fn extract_img_inn(path_to_book: &PathBuf, resolution: u32, path_to_thumbnail: &PathBuf) -> Result<(), MuToolError> {
  let mut child = Command::new("mutool")
    .arg("draw")
    .arg("-r")
    .arg(resolution.to_string())
    .arg("-F")
    .arg("png")
    .arg("-o")
    .arg(path_to_thumbnail)
    .arg(path_to_book)
    .arg("1")
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .spawn()
    .unwrap();
  let status = child.wait().await.unwrap();
  MuToolError::from_process_exit_status(status)
}
async fn _save_thumbnail(path_to_thumbnail: &PathBuf) {
  let data = _imp_to_jpeg(path_to_thumbnail).unwrap();
  tokio::fs::remove_file(path_to_thumbnail).await.unwrap();
  tokio::fs::write(path_to_thumbnail.with_extension("jpeg"), data).await.unwrap();
}
fn _imp_to_jpeg(path_to_thumbnail: &PathBuf) -> Result<Vec<u8>, JxlEncodeErrors> {
  let reader = BufReader::new(File::open(path_to_thumbnail).unwrap());
  let image = image::load(reader, ImageFormat::Png).unwrap();
  let (w, h) = image.dimensions();
  let img_pixels = image.as_rgb8().unwrap();
  let opts = EncoderOptions::new(w as usize, h as usize, ColorSpace::RGB, BitDepth::Eight);
  let jxl_encoder = JxlSimpleEncoder::new(img_pixels, opts);
  jxl_encoder.encode()
}
