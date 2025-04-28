use serde::{Deserialize, Serialize};


pub(crate) type BookPath = String;
pub(crate) type BookSize = String;
pub(crate) type FileSize = String;
pub(crate) type BookHash = String;
pub(crate) type NotifyEvents = notify::Result<notify::Event>;
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MutoolErr {
  SIGSEGV,
  MutoolProcessFailed,
  OtherErr,
}

#[derive(Debug)]
pub enum Error {
  PathToDataDirIsNone,
}
