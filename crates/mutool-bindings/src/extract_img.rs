use crate::MuToolResult;
use anyhow::Result;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;
use utils::get_file_size;

pub async fn extract_img(path_to_book: &PathBuf, resolution: u32, path_to_thumbnail: &PathBuf) -> Result<MuToolResult> {
  let book_size = get_file_size(path_to_book)?;
  match book_size == 0 {
    true => Err(anyhow::anyhow!("Book file is empty")),
    false => {
      match path_to_book.exists() {
        false => Err(anyhow::anyhow!("Book file not found")),
        true => {
          let mut child = Command::new("mutool")
            .arg("draw").arg("-r")
            .arg(resolution.to_string()).arg("-F")
            .arg("png").arg("-o").arg(path_to_thumbnail)
            .arg(path_to_book).arg("1")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;

          let status = child.wait().await?;
          Ok(MuToolResult::from_process_exit_status(status))
        },
      }
    },
  }
}
