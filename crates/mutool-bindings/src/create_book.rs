use crate::MuToolResult;
use anyhow::{anyhow, Result};
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;


pub async fn create_empty_book(path_to_mutool: &PathBuf, path_to_book: &PathBuf) -> Result<MuToolResult> {
  match path_to_mutool.is_file() {
    true => {
      let mut child = Command::new(path_to_mutool)
        .arg("create")
        .arg("-o").arg(path_to_book)
        .arg(if cfg!(windows) {
          "NUL"
        } else {
          "/dev/null"
        })
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

      let status = child.wait().await?;
      Ok(MuToolResult::from_process_exit_status(status))
    }
    false => {
      println!("{:?}", path_to_mutool);
      Err(anyhow!("mutool not found"))
    }
  }
}
