use crate::MUToolResult;
use crate::MUToolResult::{OtherErr, Success, SIGSEGV};
use anyhow::Result;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;
use utils::get_file_size;

async fn file_is_empty(path: &PathBuf) -> Result<bool> {
  let size = get_file_size(path)?;
  Ok(size == 0)
}

pub async fn extract_img(path_to_book: &PathBuf, resolution: u32, path_to_thumbnail: &PathBuf) -> Result<MUToolResult> {
  match file_is_empty(path_to_book).await? {
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

          match status.success() {
            true => Ok(Success),
            false => {
              #[cfg(unix)]
              {
                use std::os::unix::process::ExitStatusExt;
                match status.signal() {
                  Some(signal) if signal == libc::SIGSEGV => {
                    Ok(SIGSEGV)
                  }
                  _ => Ok(OtherErr),
                }
              }

              #[cfg(windows)]
              {
                match status.code() {
                  Some(code) if code == 0xC0000005u32 as i32 => {
                    Ok(Status::SIGSEGV)
                  }
                  _ => Ok(Status::OtherErr),
                }
              }

              #[cfg(not(any(unix, windows)))]
              {
                Ok(Status::OtherErr)
              }
            }
          }
        },
      }
    },
  }
}
