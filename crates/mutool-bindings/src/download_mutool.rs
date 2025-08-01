use anyhow::{Context, Result};
use std::io::Write;
use std::path::PathBuf;
use std::{fs, sync::{Arc, RwLock}};
use tempfile::NamedTempFile;
use tokio_stream::StreamExt;
use zip::ZipArchive;

//noinspection RsUnwrap
pub async fn download_mutool(zip_url: &str, target_dir: &PathBuf, progress: Arc<RwLock<f32>>) -> Result<()> {
  let response = reqwest::get(zip_url).await.context("Failed to perform a GET questioning")?;
  let total_size = response.content_length().context("The server did not report the amount of content")?;
  let mut temp_file = NamedTempFile::new().context("Failed to create a temporary file")?;

  let mut stream = response.bytes_stream();
  let mut downloaded: u64 = 0;

  while let Some(chunk) = stream.next().await {
    let chunk = chunk.context("Error when receiving a chink")?;
    temp_file.write_all(&chunk).context("Error when recording a chink into a temporary file")?;
    downloaded += chunk.len() as u64;

    let mut prog = progress.write().unwrap();
    *prog = (downloaded as f64 / total_size as f64) as f32;
  }
  temp_file.flush()?;
  let mut archive = ZipArchive::new(temp_file.reopen()?)?;

  for i in 0..archive.len() {
    let mut file = archive.by_index(i)?;
    let name = file.name().replace('\\', "/");
    if name.ends_with("mutool.exe") {
      fs::create_dir_all(target_dir)?;
      let out_path = target_dir.join("mutool.exe");
      let mut out_file = fs::File::create(&out_path)?;
      std::io::copy(&mut file, &mut out_file)?;
      return Ok(());
    }
  }

  Err(anyhow::anyhow!("mutool.exe not found in the archive"))
}
