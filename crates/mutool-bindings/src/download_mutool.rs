use anyhow::{Context, Result};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use tempfile::NamedTempFile;
use tokio::fs as tokio_fs;
use tokio_stream::StreamExt;

use indicatif::{ProgressBar, ProgressStyle};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

//noinspection RsUnwrap
pub async fn download_mutool(target_dir: &PathBuf, progress: Arc<RwLock<f32>>) -> Result<()> {
  #[cfg(target_os = "windows")]
  let url = "https://dw.uptodown.net/dwn/RvVkii134Riphftvun7hQBZyU0aCwJjJMFI3FD3XyiSl7SJevwENyD0jcXhKYhV4CH_qruGqhacLd-aUefTwe9zMDyaZZaCB0DAsBlBNsHh60asEu7ao6dZq9ivOTbIO/sAY2_Qg6XkK8aAMV8-eZCZlUKuoc9OWpW7QRoSl77rD4qiPKT6XxUQu6F9ugodwtN2WtM6byM_Jd2ielaIbSabs6zeNhBnGYQ5Ru50F6EqcttmsERNy3yEYcDbNxBxfu/zryZ2lIUcxdledxtl0F3OVJfSn8NZRroKmjE_IPkVeZ5FGSY5neb0gBe8tkOIwAn/mupdf-1-26-0.zip";
  #[cfg(target_os = "linux")]
  let url = "https://github.com/m59peacemaker/mupdf-appimage/releases/download/1.18.0/mutool-1.18.0-x86_64.AppImage";

  let response = reqwest::get(url).await.context("Failed to perform a GET questioning")?;
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

  #[cfg(target_os = "windows")]
  {
    let mut archive = zip::ZipArchive::new(temp_file.reopen()?)?;
    for i in 0..archive.len() {
      let mut file = archive.by_index(i)?;
      let name = file.name().replace('\\', "/");
      if name.ends_with("mutool.exe") {
        tokio_fs::create_dir_all(target_dir).await?;
        let out_path = target_dir.join("mutool.exe");
        let mut out_file = tokio::fs::File::create(&out_path).await?;
        let mut buffer = Vec::new();
        std::io::copy(&mut file, &mut buffer)?;
        tokio::io::AsyncWriteExt::write_all(&mut out_file, &buffer).await?;
        return Ok(());
      }
    }
    Err(anyhow::anyhow!("mutool.exe not found in the archive"))
  }

  #[cfg(target_os = "linux")]
  {
    tokio_fs::create_dir_all(target_dir).await?;
    let out_path = target_dir.join("mutool");
    let mut out_file = tokio_fs::File::create(&out_path).await?;
    let mut temp_reader = temp_file.reopen()?;
    let mut buffer = Vec::new();
    std::io::copy(&mut temp_reader, &mut buffer)?;
    tokio::io::AsyncWriteExt::write_all(&mut out_file, &buffer).await?;

    #[cfg(unix)]
    {
      let perms = std::fs::Permissions::from_mode(0o755);
      tokio_fs::set_permissions(&out_path, perms).await.context("Failed to set executable permission")?;
    }

    Ok(())
  }

  #[cfg(not(any(target_os = "windows", target_os = "linux")))]
  {
    Err(anyhow::anyhow!("Unsupported platform"))
  }
}

pub async fn show_download_progress(progress: Arc<RwLock<f32>>) {
  let pb = ProgressBar::new(100);
  pb.set_style(ProgressStyle::with_template("{msg}\n[{elapsed_precise}] [{wide_bar:.green/white}] {percent}%").unwrap()
    .progress_chars("=>-"));
  pb.set_message("Downloading mutool...");

  loop {
    let p = *progress.read().unwrap();
    pb.set_position((p * 100.0).round() as u64);
    if p >= 1.0 {
      break;
    }
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
  }

  pb.finish_with_message("Download complete ✅");
}

pub async fn download_mutool_if_missing_blocking(path_to_mutool_storage: &PathBuf) -> Result<()> {
  match path_to_mutool_storage.exists() {
    true => {
      let path_to_mutool: PathBuf;
      if cfg!(windows) {
        path_to_mutool = path_to_mutool_storage.join("mutool.exe");
      } else {
        path_to_mutool = path_to_mutool_storage.join("mutool");
      }
      match path_to_mutool.exists() {
        true => {}
        false => {
          let mutool_download_progress = Arc::new(RwLock::new(0.0));
          let progress_task = tokio::spawn(show_download_progress(mutool_download_progress.clone()));
          download_mutool(path_to_mutool_storage, mutool_download_progress).await?;
          progress_task.await?;

          #[cfg(target_os = "windows")]
          println!("✅ mutool.exe loaded in {:?}", path_to_mutool_storage.join("mutool.exe"));
          #[cfg(target_os = "linux")]
          println!("✅ mutool loaded in {:?}", path_to_mutool_storage.join("mutool"));
        }
      }
    }
    false => {}
  }
  Ok(())
}
