use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub enum MuToolError {
  SIGSEGV,
  FileIsEmpty,
  IoError,
  OtherErr,
  MutoolNotFound,
  ParseError(String),
  JsonError(String),
  CommandFailed(String),
}

impl fmt::Display for MuToolError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::SIGSEGV => write!(f, "mutool process segmentation fault (SIGSEGV)"),
      Self::FileIsEmpty => write!(f, "document file is empty"),
      Self::IoError => write!(f, "I/O error during mutool operation"),
      Self::OtherErr => write!(f, "unknown mutool error"),
      Self::MutoolNotFound => write!(f, "mutool executable not found on system"),
      Self::ParseError(msg) => write!(f, "failed to parse mutool output: {msg}"),
      Self::JsonError(msg) => write!(f, "failed to deserialize mutool JSON: {msg}"),
      Self::CommandFailed(msg) => write!(f, "mutool command execution failed: {msg}"),
    }
  }
}

impl std::error::Error for MuToolError {}

impl MuToolError {
  pub(crate) fn from_process_exit_status(status: std::process::ExitStatus) -> Result<(), Self> {
    match status.success() {
      true => Ok(()),
      false => {
        #[cfg(unix)]
        {
          use std::os::unix::process::ExitStatusExt;
          match status.signal() {
            Some(signal) if signal == libc::SIGSEGV => Err(Self::SIGSEGV),
            _ => Err(Self::OtherErr),
          }
        }

        #[cfg(windows)]
        {
          match status.code() {
            Some(code) if code == 0xC0000005u32 as i32 => Err(Self::SIGSEGV),
            _ => Err(Self::OtherErr),
          }
        }

        #[cfg(not(any(unix, windows)))]
        {
          Err(Self::OtherErr)
        }
      }
    }
  }
}
