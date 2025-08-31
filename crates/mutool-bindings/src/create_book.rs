use crate::mutool_status::MuToolError;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;

pub async fn create_empty_book(path_to_mutool: &PathBuf, path_to_book: &PathBuf) -> Result<(), MuToolError> {
  match path_to_mutool.is_file() {
    true => {
      let mut child = Command::new(path_to_mutool)
        .arg("create")
        .arg("-o")
        .arg(path_to_book)
        .arg(if cfg!(windows) { "NUL" } else { "/dev/null" })
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();

      let status = child.wait().await.unwrap();
      MuToolError::from_process_exit_status(status)
    }
    false => Err(MuToolError::MutoolNotFound),
  }
}
