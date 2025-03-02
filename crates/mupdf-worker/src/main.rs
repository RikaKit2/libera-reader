use glob_structs::IpcMsg;
use ipc_channel::ipc;
use ipc_channel::ipc::IpcSender;
use mupdf::document::Document;
use std::{env, process};


fn extract_thumbnail_from_doc(not_cached_book: String) {
  match Document::open(&not_cached_book, 20) {
    Ok(doc) => {
      let page = doc.load_page(0).unwrap();
      match page.to_pixmap(0.4) {
        Ok(mut pixmap) => {
          // let out_file_name = not_cached_book.get_out_file_name();
          // pixmap.save_as_jpeg(70, format!("{}.jpeg", out_file_name));
          // not_cached_book.mark_as_cached();
        }
        Err(_err) => {}
      };
    }
    Err(_e) => {}
  }
}

fn main() {
  let args: Vec<String> = env::args().collect();
  let token = args.get(1).expect("missing argument");

  let sender_to_main: IpcSender<IpcMsg> = IpcSender::connect(token.to_string()).expect("connect failed");
  let (tx, receiver_from_main) = ipc::channel().unwrap();
  sender_to_main.send(IpcMsg::IpcSender(tx)).expect("send failed");
  loop {
    match receiver_from_main.try_recv() {
      Ok(msg) => {
        match msg {
          IpcMsg::BookToCaching(String) => {
            
          }
          IpcMsg::ExitSignalForMupdf => { process::exit(0); }
          _ => {}
        }
      }
      Err(_) => {}
    }
  }
}
