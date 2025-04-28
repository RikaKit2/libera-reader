use crate::models::Book;
use crate::services::Status::{NotWorking, Working};
use crate::services::Status;
use crate::types::MutoolErr;
use crate::utils::{get_num_of_threads, RayonTaskType::ImgExtract};
use crate::vars;
use crate::vars::NOT_CACHED_BOOKS;
use rayon::ThreadPoolBuilder;
use std::io::{Error, ErrorKind};
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
}

impl DataExtractionService {
  pub(crate) fn run(&mut self) {
    match &self.status {
      Working => {}
      NotWorking => {
        Self::run_thread();
        self.status = Working
      }
    }
  }
  pub(crate) fn stop(&mut self) {
    match &self.status {
      Working => {
        self.status = NotWorking;
        vars::SHUTDOWN.swap(false, Ordering::Relaxed);
      }
      NotWorking => {}
    }
  }
  fn run_mutool_process(not_cached_book: &Book, resolution: u32) -> Result<Child, Error> {
    match Path::new(&not_cached_book.path_to_book).exists() {
      true => {
        let child = Command::new("mutool")
          .arg("draw").arg("-r")
          .arg(resolution.to_string()).arg("-F")
          .arg("png").arg("-o")
          .arg(not_cached_book.get_path_to_storage())
          .arg(&not_cached_book.path_to_book).arg("1")
          .stdout(Stdio::null())
          .stderr(Stdio::null()).spawn()?;
        Ok(child)
      }
      false => Err(Error::new(ErrorKind::NotFound, "File not found")),
    }
  }
  fn process_result_of_mutool(not_cached_book: Box<Book>, resolution: u32) -> Result<Box<Book>, (MutoolErr, Box<Book>)> {
    let mut child = Self::run_mutool_process(&not_cached_book, resolution).unwrap();
    match child.wait() {
      Ok(status) => match status.success() {
        true => Ok(not_cached_book),
        false => match status.signal() {
          None => {
            eprintln!("mutool failed for {}: {}", &not_cached_book.path_to_book, status);
            Err((MutoolErr::OtherErr, not_cached_book))
          }
          Some(signal) => match signal.eq(&libc::SIGSEGV) {
            true => {
              eprintln!("Document caused segfault: {}", &not_cached_book.path_to_book);
              Err((MutoolErr::SIGSEGV, not_cached_book))
            }
            false => Err((MutoolErr::OtherErr, not_cached_book)),
          },
        },
      },
      Err(e) => {
        eprintln!("Failed to wait for child process: {}", e);
        Err((MutoolErr::MutoolProcessFailed, not_cached_book))
      }
    }
  }
  fn extract_thumbnails() -> Vec<Result<Box<Book>, (MutoolErr, Box<Book>)>> {
    let now = Instant::now();
    let num_workers = num_cpus::get();
    println!("Using {} worker threads", num_workers);

    let mutool_results: Vec<Result<Box<Book>, (MutoolErr, Box<Book>)>> = tokio::runtime::Builder::new_multi_thread()
      .worker_threads(num_workers)
      .enable_all()
      .build()
      .unwrap()
      .block_on(async {
        let mut mutool_join_handlers = Vec::new();
        let mut mutool_results = Vec::new();
        let semaphore = Arc::new(Semaphore::new(num_workers));

        for not_cached_book in NOT_CACHED_BOOKS.try_iter() {
          let semaphore = Arc::clone(&semaphore);
          mutool_join_handlers.push(tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();
            return Self::process_result_of_mutool(not_cached_book, 20);
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
  fn run_thread() {
    thread::spawn(|| {
      let num_of_threads = get_num_of_threads(ImgExtract);
      debug!("Number of threads for data extraction service: {:?}", &num_of_threads);
      ThreadPoolBuilder::new().num_threads(num_of_threads).build().unwrap().install(|| loop {
        match vars::SHUTDOWN.load(Ordering::Relaxed) {
          true => { break; }
          false => {
            if NOT_CACHED_BOOKS.len() > 0 {
              for res in Self::extract_thumbnails() {
                match res {
                  Ok(cached_book) => { cached_book.mark_as_cached(); }
                  Err((err, broken_book)) => { broken_book.mark_as_broken(err); }
                }
              }
            }
          }
        }
      });
    });
  }
}
