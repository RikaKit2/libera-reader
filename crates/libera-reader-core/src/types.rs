use crate::db::models::BookDataType;

pub(crate) type BookPath = String;
pub(crate) type BookSize = String;
pub(crate) type BookHash = String;
pub(crate) type NotifyEvents = notify::Result<notify::Event>;

pub(crate) struct NotCachedBook {
  pub data_type: BookDataType,
  pub path_to_book: BookPath,
}
