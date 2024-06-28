use std::collections::VecDeque;
use std::fs;
use std::hash::{BuildHasher, Hasher};
use std::io::Read;
use std::path::PathBuf;
use std::sync::Arc;

use gxhash::GxBuildHasher;
use tokio::sync::RwLock;

use crate::app_state::NotCachedBook;
use crate::db::crud;
use crate::db::prisma::prisma::PrismaClient;

fn round_num(x: f64, decimals: u32) -> f64 {
  let y = 10i32.pow(decimals) as f64;
  (x * y).round() / y
}

fn calc_file_size_in_mb(path_to_file: &String) -> f64 {
  let metadata = fs::metadata(path_to_file).unwrap();
  let size_mb = metadata.len() as f64 / (1024.0 * 1024.0);
  round_num(size_mb, 2)
}

fn calc_gxhash_of_file(path_to_file: &String) -> String {
  let chunk_size = 1024 * 1024; // 1 MB
  let mut hasher = GxBuildHasher::default().build_hasher();
  let mut file = fs::File::open(path_to_file).unwrap();
  loop {
    let mut buffer = vec![0; chunk_size];
    let bytes_read = file.read(&mut buffer).unwrap();
    if bytes_read == 0 {
      break;
    }
    hasher.write(&buffer[..bytes_read]);
  }

  let hash = hasher.finish().to_string();
  hash
}

pub async fn create_new_book(path_to_book: String, book_folder: String, book_name: String,
                             ext: String, target_ext: &Arc<RwLock<gxhash::HashSet<String>>>,
                             not_cached_books: &Arc<RwLock<VecDeque<NotCachedBook>>>,
                             client: &Arc<PrismaClient>) {
  if target_ext.read().await.contains(&ext) {
    let book_hash = calc_gxhash_of_file(&path_to_book);
    match crud::get_book_data(book_hash.clone(), client).await {
      None => {
        let file_size = calc_file_size_in_mb(&path_to_book);
        let book_data = crud::create_book_data(book_hash.clone(), file_size, ext, client).await;
        crud::create_book_item(path_to_book.clone(), book_data.id, book_folder, book_name, client).await;
        not_cached_books.write().await.push_front(NotCachedBook { book_hash, path_to_book });
      }
      Some(book_data) => {
        crud::create_book_item(path_to_book, book_data.id, book_folder, book_name, client).await;
      }
    }
  }
}


pub async fn rename_book_data(new_path: &PathBuf, old_path: &PathBuf, client: &PrismaClient) {
  let new_path_to_book = new_path.to_str().unwrap().to_string();
  let old_path_to_book = old_path.to_str().unwrap().to_string();

  let new_dir = new_path.parent().unwrap().to_str().unwrap().to_string();
  let old_dir = old_path.parent().unwrap().to_str().unwrap().to_string();
  let dir_equals = new_dir == old_dir;
  if dir_equals == false {
    crud::change_path_and_dir(new_path_to_book, old_path_to_book, new_dir, client).await;
    return;
  }

  let new_book_name = new_path.file_name().unwrap().to_str().unwrap().to_string();
  let old_book_name = old_path.file_name().unwrap().to_str().unwrap().to_string();
  let book_name_equals = new_book_name == old_book_name;
  if book_name_equals == false {
    crud::change_path_and_book_name(new_path_to_book, old_path_to_book, new_book_name, client).await;
    return;
  }

  let new_ext = new_path.extension().unwrap().to_str().unwrap().to_string();
  let old_ext = old_path.extension().unwrap().to_str().unwrap().to_string();
  let ext_equals = new_ext == old_ext;
  if ext_equals == false {
    crud::change_path_and_ext(new_path_to_book, old_path_to_book, new_ext, client).await;
  }
}

pub async fn delete_book(path_to_book: String, client: &PrismaClient) {
  match crud::get_book_item(path_to_book, client).await {
    None => {}
    Some(book_item) => {
      let num_links_per_book_data = crud::get_num_links_per_book_data(book_item.id, client).await;
      if num_links_per_book_data == 1 {
        crud::delete_book_data(book_item.book_data_id, client).await;
        crud::delete_book_item(book_item.id, client).await;
      } else if num_links_per_book_data > 1 {
        crud::delete_book_item(book_item.id, client).await;
      }
    }
  }
}

pub async fn delete_dir(paths: Vec<PathBuf>, client: &PrismaClient) {
  let path_to_dir = paths[0].to_str().unwrap().to_string();
  let outdated_books = crud::get_books_contains_path_to_dir(
    path_to_dir, client).await;
  for outdated_book in outdated_books {
    delete_book(outdated_book.path_to_book, client).await;
  }
}

pub async fn rename_dir(new_path: &PathBuf, old_path: &PathBuf, client: &PrismaClient) {
  let new_path_to_dir = new_path.to_str().unwrap();
  let old_path_to_dir = old_path.to_str().unwrap();
  let new_dir = new_path.parent().unwrap().to_str().unwrap().to_string();

  let old_paths_to_books = crud::get_books_contains_path_to_dir(
    old_path_to_dir.to_string(), client).await;

  for book_item in old_paths_to_books {
    let old_path_to_book = book_item.path_to_book;
    let new_path_to_book = PathBuf::from(new_path_to_dir)
      .join(book_item.file_name).to_str().unwrap().to_string();
    crud::change_path_and_dir(new_path_to_book, old_path_to_book, new_dir.clone(), client).await;
  }
}
