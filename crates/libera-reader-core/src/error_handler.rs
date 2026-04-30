use chrono::{DateTime, Local};
use gpui::SharedString;
use tokio::sync::mpsc::{self, UnboundedReceiver};

#[derive(Debug)]
pub struct ErrorEntry {
  pub timestamp: DateTime<Local>,
  pub message: SharedString,
  pub error_type: ErrorType,
}

#[derive(Debug)]
pub enum ErrorType {
  DB,
  FileSystem,
  Network,
  Other,
}

pub struct ErrorReceiver(UnboundedReceiver<ErrorEntry>);
impl ErrorReceiver {
  pub(crate) fn new(rx: UnboundedReceiver<ErrorEntry>) -> Self {
    Self(rx)
  }
  pub async fn recv(&mut self) -> Option<ErrorEntry> {
    self.0.recv().await
  }
}
#[derive(Clone)]
pub struct ErrorHandler {
  sender: mpsc::UnboundedSender<ErrorEntry>,
}

impl ErrorHandler {
  pub fn new() -> (Self, ErrorReceiver) {
    let (tx, rx) = mpsc::unbounded_channel();
    (Self { sender: tx }, ErrorReceiver::new(rx))
  }

  pub fn report(&self, message: SharedString, error_type: ErrorType) {
    self.sender.send(ErrorEntry { timestamp: Local::now(), message, error_type }).unwrap();
  }
}

pub trait ErrorHandlerExt<T, E> {
  fn or_report(self, handler: &ErrorHandler, error: ErrorType) -> Option<T>;
}

impl<T, E: std::fmt::Display> ErrorHandlerExt<T, E> for Result<T, E> {
  fn or_report(self, handler: &ErrorHandler, error_type: ErrorType) -> Option<T> {
    match self {
      Ok(val) => Some(val),
      Err(e) => {
        handler.report(e.to_string().into(), error_type);
        None
      }
    }
  }
}
