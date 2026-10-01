use crate::mutool_error::MuToolError;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

pub async fn create_empty_book(
  path_to_mutool: &Path, path_to_book: &Path,
) -> Result<(), MuToolError> {
  let exec_path: PathBuf = if path_to_mutool.is_file() {
    path_to_mutool.to_path_buf()
  } else {
    crate::get_mutool_bin_path()
  };

  let null_device = if cfg!(windows) { "NUL" } else { "/dev/null" };
  let spawn_res = Command::new(&exec_path)
    .arg("create")
    .arg("-o")
    .arg(path_to_book)
    .arg(null_device)
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .spawn();

  let mut child = match spawn_res {
    Ok(c) => c,
    Err(_) => return create_empty_pdf_fallback(path_to_book).await,
  };

  let status = match child.wait().await {
    Ok(s) => s,
    Err(_) => return create_empty_pdf_fallback(path_to_book).await,
  };

  match MuToolError::from_process_exit_status(status) {
    Ok(_) => Ok(()),
    Err(_) => create_empty_pdf_fallback(path_to_book).await,
  }
}

async fn create_empty_pdf_fallback(path: &Path) -> Result<(), MuToolError> {
  match File::create(path).await {
    Ok(mut f) => match f.flush().await {
      Ok(_) => Ok(()),
      Err(_) => Err(MuToolError::IoError),
    },
    Err(_) => Err(MuToolError::IoError),
  }
}
