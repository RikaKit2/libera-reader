use crate::models::Book;
use crate::utils::RayonTaskType::ImgExtract;
use crate::utils::{get_num_of_threads, NotCachedBook};
use crate::vars::NOT_CACHED_BOOKS;
use glob_structs::IpcMsg;
use gxhash::HashSet;
use ipc_channel::ipc;
use ipc_channel::ipc::{IpcOneShotServer, IpcReceiver, TryRecvError};
use mupdf::document::Document;
use rayon::prelude::*;
use rayon::ThreadPoolBuilder;
use std::process;
use std::process::{Child, ExitStatus};
use std::thread::sleep;
use std::time::Duration;
use tracing::debug;


pub(crate) fn fill_storage_of_non_cached_books(general_books: HashSet<Book>) {
  for i in general_books {
    if !i.get_book_data().cached {
      NotCachedBook::new(i.path_to_book).push_to_storage();
    }
  }
  debug!("Number of NOT_CACHED_BOOKS: {:?}", &NOT_CACHED_BOOKS.len());
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
                pixmap.save_as_jpeg_to_storage(70, format!("{}.jpeg", out_file_name));
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

fn msg_processing_by_mupdf(receiver_from_mupdf: &IpcReceiver<IpcMsg>) {
  match receiver_from_mupdf.try_recv() {
    Ok(ipc_msg) => {
      match ipc_msg {
        IpcMsg::BookISCaching(BookPath) => {}
        _ => {}
      }
    }
    Err(_) => {}
  }
}

pub(crate) fn spawn_process_mupdf() {
  let (server, token) =
    IpcOneShotServer::<IpcMsg>::new().expect("Failed to create IPC one-shot server.");

  let mut command = process::Command::new("");
  let child_process = command.arg(token);

  let mut child = child_process.spawn().expect("Failed to start child process");

  let (receiver_from_mupdf, first_ipc_msg) = server.accept().expect("accept failed");
  match first_ipc_msg {
    IpcMsg::IpcSender(sender_to_mupdf) => {
      loop {
        // check_stauts_of_mupdf_process
        match child.try_wait() {
          Ok(status) => {
            match status {
              None => { msg_processing_by_mupdf(&receiver_from_mupdf); }
              Some(code) => {}
            }
          }
          Err(err) => {}
        }
      }
    }
    _ => {}
  }
}
