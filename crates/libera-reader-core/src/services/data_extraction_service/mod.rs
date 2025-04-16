use crate::models::Book;
use crate::not_cached_book::NotCachedBook;
use crate::utils::get_num_of_threads;
use crate::utils::RayonTaskType::ImgExtract;
use crate::vars::NOT_CACHED_BOOKS;
use gxhash::HashSet;
use rayon::ThreadPoolBuilder;
use std::io::{Error, ErrorKind};
use std::os::unix::prelude::ExitStatusExt;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Semaphore;
use tracing::debug;


enum MutoolErr {
  SIGSEGV(Box<NotCachedBook>),
  ChildFailed(Box<NotCachedBook>),
  OtherFailed(Box<NotCachedBook>),
}

pub(crate) fn run() {
  let num_of_threads = get_num_of_threads(ImgExtract);
  debug!("Number of threads for data extraction service: {:?}", &num_of_threads);
  ThreadPoolBuilder::new().num_threads(num_of_threads).build().unwrap().install(|| {
    loop {
      if NOT_CACHED_BOOKS.read().unwrap().len() > 0 {
        for res in extract_thumbnails() {
          match res {
            Ok(_) => {}
            Err(err) => {
              match err {
                MutoolErr::SIGSEGV(broken_book) => {}
                _ => {}
              }
            }
          }
        };
      }
    }
  });
}
fn run_mutool_process(not_cached_book: &NotCachedBook, resolution: u32) -> Result<Child, Error> {
  match Path::new(&not_cached_book.book_path).exists() {
    true => {
      let child = Command::new("mutool").arg("draw")
        .arg("-r").arg(resolution.to_string())
        .arg("-F").arg("png")
        .arg("-o").arg(not_cached_book.get_out_file_name())
        .arg(&not_cached_book.book_path).arg("1")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
      Ok(child)
    }
    false => {
      Err(Error::new(ErrorKind::NotFound, "File not found"))
    }
  }
}
fn process_result_of_mutool(not_cached_book: Box<NotCachedBook>, resolution: u32) -> Result<(), MutoolErr> {
  let mut child = run_mutool_process(&not_cached_book, resolution).unwrap();
  match child.wait() {
    Ok(status) => {
      match status.success() {
        true => { Ok(()) }
        false => {
          match status.signal() {
            None => {
              eprintln!("mutool failed for {}: {}", &not_cached_book.book_path, status);
              Err(MutoolErr::OtherFailed(not_cached_book))
            }
            Some(signal) => {
              match signal.eq(&libc::SIGSEGV) {
                true => {
                  eprintln!("Document caused segfault: {}", &not_cached_book.book_path);
                  Err(MutoolErr::SIGSEGV(not_cached_book))
                }
                false => {
                  Err(MutoolErr::OtherFailed(not_cached_book))
                }
              }
            }
          }
        }
      }
    }
    Err(e) => {
      eprintln!("Failed to wait for child process: {}", e);
      Err(MutoolErr::ChildFailed(not_cached_book))
    }
  }
}
fn extract_thumbnails() -> Vec<Result<(), MutoolErr>> {
  let now = Instant::now();
  let num_workers = num_cpus::get();
  println!("Using {} worker threads", num_workers);

  let mutool_results: Vec<Result<(), MutoolErr>> = tokio::runtime::Builder::new_multi_thread()
    .worker_threads(num_workers)
    .enable_all()
    .build()
    .unwrap()
    .block_on(async {
      let mut mutool_join_handlers = Vec::new();
      let mut mutool_results = Vec::new();
      let semaphore = Arc::new(Semaphore::new(num_workers));

      for not_cached_book in NOT_CACHED_BOOKS.read().unwrap().iter().cloned() {
        let semaphore = Arc::clone(&semaphore);
        mutool_join_handlers.push(tokio::spawn(async move {
          let _permit = semaphore.acquire().await.unwrap();
          return process_result_of_mutool(not_cached_book, 20);
        }));
      }

      for handle in mutool_join_handlers {
        match handle.await {
          Ok(res) => { mutool_results.push(res); }
          Err(e) => { eprintln!("Task failed: {}", e); }
        }
      }
      return mutool_results;
    });

  let elapsed = now.elapsed();
  println!("service uptime: {:?}", elapsed);
  mutool_results
}
pub(crate) fn fill_storage_of_non_cached_books(general_books: HashSet<Book>) {
  for i in general_books {
    if !i.get_book_data().cached {
      NotCachedBook::new(i.path_to_book).push_to_storage();
    }
  }
  debug!("Number of NOT_CACHED_BOOKS: {:?}", &NOT_CACHED_BOOKS.read().unwrap().len());
}
