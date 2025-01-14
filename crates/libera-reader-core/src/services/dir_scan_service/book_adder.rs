use crate::services::notify_service::handlers::book_adding_handler;
use gxhash::HashSet;
use num_cpus;
use rayon::prelude::*;
use rayon::ThreadPoolBuilder;
use std::path::PathBuf;
use std::time::Duration;
use tracing::debug;


pub(crate) fn run(new_books: HashSet<PathBuf>) {
  let num_of_cpus = num_cpus::get();
  let num_threads: usize;
  if num_of_cpus >= 6 {
    num_threads = 2;
  } else if num_of_cpus == 1 {
    num_threads = 1;
  } else {
    num_threads = 1;
  }
  debug!("num of threads for the notify service: {:?}", &num_threads);

  let time_to_add_books: Vec<Duration> = ThreadPoolBuilder::new().num_threads(1).build().unwrap().install(|| {
    let time_to_add_books: Vec<Duration> = new_books.into_par_iter().map(|new_book| {
      book_adding_handler(&new_book)
    }).collect();
    time_to_add_books
  });

  let total_time: Duration = time_to_add_books.into_iter().sum();
  debug!("Time to add new books is: {:?}", total_time);
}
