use crate::services::notify_service::handlers::book_adding_handler;
use gxhash::HashSet;
use rayon::prelude::*;
use std::path::PathBuf;
use tracing::debug;


pub(crate) async fn run(new_books: HashSet<PathBuf>) {
  let start_time = std::time::Instant::now();
  new_books.par_iter().for_each(|new_book| {
    book_adding_handler(new_book);
  });
  debug!("Time to add new books is: {:?}", start_time.elapsed());
}
