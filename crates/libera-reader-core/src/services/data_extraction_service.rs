use crate::models::Book;
use crate::utils::RayonTaskType::ImgExtract;
use crate::utils::{get_num_of_threads, NotCachedBook};
use crate::vars::NOT_CACHED_BOOKS;
use gxhash::HashSet;
use mupdf::document::Document;
use rayon::prelude::*;
use rayon::ThreadPoolBuilder;
use std::thread::sleep;
use std::time::Duration;
use tracing::debug;


pub(crate) fn fill_storage_of_non_cached_books(general_books: HashSet<Book>) {
  for i in general_books {
    if !i.get_book_data().cached {
      NotCachedBook::new(i.path_to_book).push_to_storage();
    }
  }
  debug!("Number of uncached books: {:?}", &NOT_CACHED_BOOKS.len());
}


pub(crate) fn run() {
  let num_of_threads = get_num_of_threads(ImgExtract);
  debug!("Number of threads for data extraction service: {:?}", &num_of_threads);
  ThreadPoolBuilder::new().num_threads(num_of_threads).build().unwrap().install(|| {
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
              Err(_err) => {}
            };
          }
          Err(_e) => {}
        }
      });
      sleep(Duration::from_secs(1));
    }
  });
}
