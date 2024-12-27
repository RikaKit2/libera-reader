use crate::db::crud;
use crate::db::models::Book;
use crate::types::BookPath;

pub struct BookApi {}

impl BookApi {
  pub fn new() -> Self { Self {} }
  pub fn get_book_by_path(&self, path_to_book: &BookPath) -> Option<Book> {
    crud::get_primary::<Book>(path_to_book.clone())
  }
  pub fn get_books_from_db(&self) {}
}
impl Default for BookApi {
  fn default() -> Self {
    Self {}
  }
}

