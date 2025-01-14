use crate::models::Book;
use crate::utils::NotCachedBook;
use crate::vars::NOT_CACHED_BOOKS;
use gxhash::HashSet;
use mupdf::document::Document;
use rayon::prelude::*;
use rayon::ThreadPoolBuilder;
use std::thread::sleep;
use std::time::Duration;
use tracing::{debug, error};


pub(crate) fn fill_storage_of_non_cached_books(general_books: HashSet<Book>) {
  for i in general_books {
    if !i.get_book_data().cached {
      NotCachedBook::new(i.path_to_book).push_to_storage();
    }
  }
}


pub(crate) fn run() {
  let num_of_cpus = num_cpus::get();
  let num_threads: usize;
  if num_of_cpus >= 6 {
    num_threads = num_of_cpus - 2;
  } else if num_of_cpus == 1 {
    num_threads = 1;
  } else {
    num_threads = num_of_cpus - 1;
  }
  
  debug!("num of threads for the data extraction service: {:?}", &num_threads);
  ThreadPoolBuilder::new().num_threads(num_threads).build().unwrap().install(|| {
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
      sleep(Duration::from_millis(1000));
    }
  });
}
