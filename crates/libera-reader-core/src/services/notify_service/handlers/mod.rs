use crate::db::DB;
use crate::models::Book;
use itertools::Itertools;

pub(crate) mod book_path_update_handler;
pub(crate) mod book_adding_handler;
pub(crate) mod dir_renaming_handler;
pub(crate) mod dir_deletion_handler;
pub(crate) mod book_deletion_handler;

pub(crate) use book_path_update_handler::book_path_update_handler;
pub(crate) use book_adding_handler::book_adding_handler;
pub(crate) use dir_renaming_handler::dir_renaming_handler;
pub(crate) use dir_deletion_handler::dir_deletion_handler;
pub(crate) use book_deletion_handler::book_deletion_handler;

fn get_books_located_in_dir(path_to_dir: String) -> Vec<Book> {
  let r_conn = DB.r_transaction().unwrap();
  let books: Vec<Book> = r_conn.scan().primary().unwrap().start_with(path_to_dir).unwrap().try_collect().unwrap();
  books
}
