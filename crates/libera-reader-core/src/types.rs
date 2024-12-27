use crate::db::models::BookDataType;

pub(crate) type BookPath = String;
pub(crate) type BookSize = String;
pub(crate) type BookHash = String;
pub(crate) type BooksCount = usize;

pub(crate) struct NotCachedBook {
  pub data_type: BookDataType,
  pub path_to_book: String,
}
