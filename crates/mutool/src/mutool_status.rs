use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub enum MuToolError {
  SIGSEGV,
  FileIsEmpty,
  IoError,
  OtherErr,
  MutoolNotFound,
}
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
