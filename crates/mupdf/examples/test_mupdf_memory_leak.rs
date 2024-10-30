use std::collections::VecDeque;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use std::{io, thread};

use walkdir::WalkDir;

//noinspection DuplicatedCode
fn main() {
  println!("Enter the path to the folder to scan:");
  let mut user_input = String::new();
  io::stdin().read_line(&mut user_input).expect("Failed to read line");
  let path_to_scan = user_input.trim().to_string();

  let out_dir = PathBuf::new().join("out");
  if !out_dir.exists() {
    std::fs::create_dir(&out_dir).expect("TODO: panic message");
  };
  let mut books_paths_from_disk: VecDeque<String> = VecDeque::new();
  for entry in WalkDir::new(path_to_scan) {
    let entry = entry.unwrap();
    if entry.file_type().is_file() {
      let path = entry.path();
      let file_extension = path.extension().unwrap().to_str().unwrap().to_string();
      if ["pdf".to_string()].contains(&file_extension) {
        books_paths_from_disk.push_back(path.to_str().unwrap().to_string());
      }
    }
  };
  run_task(books_paths_from_disk, out_dir);
  loop {
    thread::sleep(Duration::from_secs(10));
  }
}

fn run_task(mut books_paths_on_disk: VecDeque<String>, out_dir: PathBuf) {
  let num_of_books = books_paths_on_disk.len();
  println!("num_of_books: {:?}", num_of_books);
  if num_of_books > 0 {
    let now = Instant::now();
    for book_num in 0..num_of_books {
      match books_paths_on_disk.pop_front() {
        Some(path_to_book) => {
          let out_file_name = out_dir.clone().join(book_num.to_string()).to_str().unwrap().to_string();
          extract_book_thumbnail(&path_to_book, out_file_name);
        }
        None => {}
      };
    }
    let elapsed = now.elapsed();
    println!("service uptime: {:?}", elapsed);
  }
}

fn extract_book_thumbnail(path_to_book: &String, out_file_name: String) {
  let document = mupdf::document::Document::open(path_to_book, 20);
  println!("mupdf opened the book: {:?}", &out_file_name);
  match document {
    Ok(doc) => {
      let page = doc.load_page(0).unwrap();
      match page.to_pixmap(0.4) {
        Ok(mut pixmap) => {
          pixmap.save_as_jpeg(70, format!("{}.png", out_file_name));
        }
        Err(err) => {
          println!("err: {:?}", &err);
        }
      };
    }
    Err(err) => {
      println!("err: {:?}", &err);
    }
  }
}

