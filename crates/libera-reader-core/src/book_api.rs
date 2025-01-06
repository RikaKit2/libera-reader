use crate::db::models::Book;
use crate::db::DB;
use crate::types::BookPath;
use itertools::Itertools;

pub struct BookApi {}

impl BookApi {
  pub fn new() -> Self { Self {} }
  //noinspection RsUnwrap
  pub fn get_book_by_path(&self, path_to_book: &BookPath) -> native_db::db_type::Result<Option<Book>> {
    let r_conn = DB.r_transaction().unwrap();
    r_conn.get().primary::<Book>(path_to_book.clone())
  }
  pub fn get_books_from_db(&self) -> Vec<Book> {
    let r_conn = DB.r_transaction().unwrap();
    let mut res: Vec<Book> = vec![];
    for i in r_conn.scan().primary::<Book>().unwrap().all().unwrap().collect_vec() {
      if i.is_ok() {
        res.push(i.unwrap());
      }
    }
    res
  }
}
impl Default for BookApi {
  fn default() -> Self {
    Self {}
  }
}

