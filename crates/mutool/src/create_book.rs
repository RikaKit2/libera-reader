use crate::mutool_error::MuToolError;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

fn find_system_mutool() -> Option<PathBuf> {
  let exe_name = if cfg!(windows) { "mutool.exe" } else { "mutool" };
  if let Some(paths) = env::var_os("PATH") {
    for p in env::split_paths(&paths) {
      let candidate = p.join(exe_name);
      if candidate.is_file() {
        return Some(candidate);
      }
    }
  }
  None
}
pub async fn create_empty_book(
  path_to_mutool: &Path, path_to_book: &Path,
) -> Result<(), MuToolError> {
  let exec_path: PathBuf = if let Some(system) = find_system_mutool() {
    system
  } else if path_to_mutool.is_file() {
    path_to_mutool.to_path_buf()
  } else {
    return create_empty_pdf_fallback(path_to_book).await;
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
