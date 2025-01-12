use crate::models::Book;
use crate::utils::NotCachedBook;
use crate::vars::NOT_CACHED_BOOKS;
use gxhash::HashSet;
use mupdf::document::Document;
use rayon::prelude::*;
use std::time::Duration;
use tokio::time::sleep;
use tracing::error;


pub(crate) fn fill_storage_of_non_cached_books(general_books: HashSet<Book>) {
  for i in general_books {
    NotCachedBook::new(i.book_data_pk, i.path_to_book).push_to_storage();
  }
}


pub(crate) async fn run() {
  loop {
    NOT_CACHED_BOOKS.try_iter().par_bridge().for_each(|not_cached_book| {
      match Document::open(&not_cached_book.book_path, 20) {
        Ok(doc) => {
          let page = doc.load_page(0).unwrap();
          match page.to_pixmap(0.4) {
            Ok(mut pixmap) => {
              let out_file_name = not_cached_book.get_out_file_name();
              pixmap.save_as_jpeg(70, format!("{}.jpeg", out_file_name));
              not_cached_book.mark_as_cached();
            }
            Err(err) => {
              error!("err: {:?}", &err);
            }
          };
        }
        Err(e) => { error!("{:?}", &e); }
      }
    });
    sleep(Duration::from_millis(1000)).await;
  }
}
