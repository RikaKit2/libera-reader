use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MuToolResult {
  Success,
  SIGSEGV,
  FileIsEmpty,
  OtherErr,
}
impl MuToolResult {
  pub(crate) fn from_process_exit_status(status: std::process::ExitStatus) -> Self {
    match status.success() {
      true => Self::Success,
      false => {
        #[cfg(unix)]
        {
          use std::os::unix::process::ExitStatusExt;
          match status.signal() {
            Some(signal) if signal == libc::SIGSEGV => {
              Self::SIGSEGV
            }
            _ => Self::OtherErr,
          }
        }

        #[cfg(windows)]
        {
          match status.code() {
            Some(code) if code == 0xC0000005u32 as i32 => {
              Self::SIGSEGV
            }
            _ => Self::OtherErr,
          }
        }

        #[cfg(not(any(unix, windows)))]
        {
          Self::OtherErr
        }
      }
    }
  }
}