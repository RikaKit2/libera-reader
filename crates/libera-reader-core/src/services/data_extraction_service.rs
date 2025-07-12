use crate::db::models::Book;
use crate::services::{Status, Status::{NotWorking, Working}};
use crate::types::{MutoolErr, NotCachedBooks, APP_DIRS, DB};
use crate::utils::RayonTask;
use crate::vars;
use anyhow::Result;
use rayon::ThreadPoolBuilder;
use std::io::{Error, ErrorKind};
#[cfg(unix)]
use std::os::unix::prelude::ExitStatusExt;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::thread;
use std::time::Instant;
use tokio::sync::Semaphore;
use tracing::debug;

pub struct DataExtractionService {
  pub status: Status,
  not_cached_books: NotCachedBooks,
  app_dirs: APP_DIRS,
  db: DB,
}

impl DataExtractionService {
  pub fn new(not_cached_books: NotCachedBooks, db: DB, app_dirs: APP_DIRS) -> Self {
    Self { status: NotWorking, not_cached_books, db, app_dirs }
  }
  pub(crate) fn run(&mut self) {
    match &self.status {
      Working => {}
      NotWorking => {
        Self::run_thread(self.db.clone(), self.not_cached_books.clone(), self.app_dirs.clone());
        self.status = Working
      }
    }
  }
  pub(crate) fn stop(&mut self) {
    match &self.status {
      Working => { self.status = NotWorking; }
      NotWorking => {}
    }
  }
  fn spawn_mutool_process(not_cached_book: &Book, resolution: u32, path_to_storage: String) -> Result<Child, Error> {
    match Path::new(&not_cached_book.path_to_book).exists() {
      true => {
        let child = Command::new("mutool").arg("draw").arg("-r").arg(resolution.to_string()).arg("-F").arg("png").arg("-o").arg(path_to_storage).arg(&not_cached_book.path_to_book).arg("1").stdout(Stdio::null()).stderr(Stdio::null()).spawn()?;
        Ok(child)
      }
      false => Err(Error::new(ErrorKind::NotFound, "File not found")),
    }
  }
  fn processing_the_result_from_mutool(not_cached_book: Box<Book>, resolution: u32, path_to_storage: String) -> Result<Box<Book>, (MutoolErr, Box<Book>)> {
    let mut child = Self::spawn_mutool_process(&not_cached_book, resolution, path_to_storage).unwrap();
    match child.wait() {
      Ok(status) => match status.success() {
        true => Ok(not_cached_book),
        false => {
          #[cfg(unix)]
          {
            match status.signal() {
              Some(signal) if signal == libc::SIGSEGV => {
                eprintln!("Document caused segfault: {}", &not_cached_book.path_to_book);
                Err((MutoolErr::SIGSEGV, not_cached_book))
              }
              _ => {
                eprintln!("mutool failed for {}: {}", &not_cached_book.path_to_book, status);
                Err((MutoolErr::OtherErr, not_cached_book))
              }
            }
          }

          #[cfg(windows)]
          {
            match status.code() {
              Some(code) if code == 0xC0000005u32 as i32 => {
                eprintln!("Document caused access violation: {}", &not_cached_book.path_to_book);
                Err((MutoolErr::SIGSEGV, not_cached_book))
              }
              _ => {
                eprintln!("mutool failed for {}: {}", &not_cached_book.path_to_book, status);
                Err((MutoolErr::OtherErr, not_cached_book))
              }
            }
          }

          #[cfg(not(any(unix, windows)))]
          {
            eprintln!("mutool failed for {}: {}", &not_cached_book.path_to_book, status);
            Err((MutoolErr::OtherErr, not_cached_book))
          }
        }
      },
      Err(e) => {
        eprintln!("Failed to wait for child process: {}", e);
        Err((MutoolErr::MutoolProcessFailed, not_cached_book))
      }
    }
  }
  fn extract_thumbnails(not_cached_books: &NotCachedBooks, app_dirs: &APP_DIRS) -> Vec<Result<Box<Book>, (MutoolErr, Box<Book>)>> {
    let now = Instant::now();
    let num_workers = num_cpus::get();
    println!("Using {} worker threads", num_workers);

    let mutool_results: Vec<Result<Box<Book>, (MutoolErr, Box<Book>)>> = tokio::runtime::Builder::new_multi_thread().worker_threads(num_workers).enable_all().build().unwrap().block_on(async {
      let mut mutool_join_handlers = Vec::new();
      let mut mutool_results = Vec::new();
      let semaphore = Arc::new(Semaphore::new(num_workers));

      for not_cached_book in not_cached_books.try_iter() {
        let path_to_storage = not_cached_book.get_path_to_storage(app_dirs);
        let semaphore = Arc::clone(&semaphore);
        mutool_join_handlers.push(tokio::spawn(async move {
          let _permit = semaphore.acquire().await.unwrap();
          return Self::processing_the_result_from_mutool(not_cached_book, 20, path_to_storage);
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
  fn run_thread(db: DB, not_cached_books: NotCachedBooks, app_dirs: APP_DIRS) {
    thread::spawn(move || {
      let num_of_threads = RayonTask::ExtractImg.get_num_of_threads();
      debug!("Number of threads for data extraction service: {:?}", &num_of_threads);
      ThreadPoolBuilder::new().num_threads(num_of_threads).build().unwrap().install(|| loop {
        match vars::SHUTDOWN.load(Ordering::Relaxed) {
          true => { break; }
          false => {
            if not_cached_books.len() > 0 {
              for res in Self::extract_thumbnails(&not_cached_books, &app_dirs) {
                match res {
                  Ok(cached_book) => { let _ = cached_book.mark_as_cached(&db); }
                  Err((err, broken_book)) => { let _ = broken_book.mark_as_broken(err, &db); }
                }
              }
            }
          }
        }
      });
    });
  }
}

