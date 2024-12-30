use crate::db::models::BookDataType;
use crossbeam_channel::Receiver;
use notify::RecommendedWatcher;
use std::sync::{Arc, Mutex};

pub(crate) type BookPath = String;
pub(crate) type BookSize = String;
pub(crate) type BookHash = String;
pub(crate) type BooksCount = usize;
pub(crate) type NotifyEvents = Receiver<notify::Result<notify::Event>>;
pub(crate) type NotifyWatcher = Arc<Mutex<RecommendedWatcher>>;

pub(crate) struct NotCachedBook {
  pub data_type: BookDataType,
  pub path_to_book: String,
}
