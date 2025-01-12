use crate::db::crud;
use crate::models::Book;


pub(crate) fn del_outdated_books(outdated_books: Vec<Book>) {
  for outdated_book in outdated_books {
    crud::book::del_book_and_its_data(outdated_book);
  }
}