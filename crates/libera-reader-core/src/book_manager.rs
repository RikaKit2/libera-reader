use std::collections::{HashSet, VecDeque};
use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::RwLock;
use tracing::info;

use crate::db::crud;
use crate::db::model::PrismaClient;
use crate::utils::{calc_blake3_hash_of_file, calc_file_size_in_mb};
use crate::utils::NotCachedBook;

pub async fn create_new_book(path: &PathBuf, target_ext: &Arc<RwLock<HashSet<String>>>,
                             not_cached_books: &Arc<RwLock<VecDeque<NotCachedBook>>>,
                             client: &PrismaClient) {
  let ext = path.extension().unwrap().to_str().unwrap().to_string();
  if target_ext.read().await.contains(&ext) {
    let path_to_book = path.to_str().unwrap().to_string();
    let book_folder = path.parent().unwrap().to_str().unwrap().to_string();
    let book_name = path.file_name().unwrap().to_str().unwrap().to_string();
    info!("\n create new book:\n{:?}", &path_to_book);
    let book_hash = calc_blake3_hash_of_file(&path_to_book);
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

pub async fn rename_book_data(old_path: &PathBuf, new_path: &PathBuf, client: &PrismaClient,
                              target_ext: &Arc<RwLock<HashSet<String>>>,
                              not_cached_books: &Arc<RwLock<VecDeque<NotCachedBook>>>) {
  let old_path_to_book = old_path.to_str().unwrap().to_string();
  let new_path_to_book = new_path.to_str().unwrap().to_string();
  match crud::get_book_item(old_path_to_book, client).await {
    None => {
      create_new_book(new_path, target_ext, not_cached_books, client).await;
    }
    Some(old_book) => {
      info!("\n rename book data\n new_path:\n{:?}\n old_path:\n{:?}", &new_path_to_book, &old_book.path_to_book);

      let new_dir = new_path.parent().unwrap().to_str().unwrap().to_string();
      let old_dir = old_path.parent().unwrap().to_str().unwrap().to_string();
      let new_book_name = new_path.file_name().unwrap().to_str().unwrap().to_string();
      let old_book_name = old_path.file_name().unwrap().to_str().unwrap().to_string();
      let new_ext = new_path.extension().unwrap().to_str().unwrap().to_string();
      let old_ext = old_path.extension().unwrap().to_str().unwrap().to_string();

      let dir_equals = new_dir == old_dir;
      let book_name_equals = new_book_name == old_book_name;
      let ext_equals = new_ext == old_ext;

      if dir_equals == false {
        crud::change_path_and_dir(new_path_to_book, old_book.path_to_book, new_dir, client).await;
        return;
      } else if book_name_equals == false {
        crud::change_path_and_book_name(new_path_to_book, old_book.path_to_book, new_book_name, client).await;
        return;
      } else if ext_equals == false {
        crud::change_path_and_ext(new_path_to_book, old_book.path_to_book, new_ext, client).await;
      }
    }
  }
}

pub async fn delete_book(old_path_to_book: String, client: &PrismaClient) {
  match crud::get_book_item(old_path_to_book.clone(), client).await {
    None => {}
    Some(book_item) => {
      let num_links_per_book_data = crud::get_num_links_per_book_data(book_item.book_data_id, client).await;
      if num_links_per_book_data == 1 {
        crud::delete_book_item(book_item.id, client).await;
        crud::delete_book_data(book_item.book_data_id, client).await;
      } else if num_links_per_book_data > 1 {
        crud::delete_book_item(book_item.id, client).await;
      }
    }
  }
}

pub async fn delete_dir(paths: Vec<PathBuf>, client: &PrismaClient) {
  let path_to_dir = paths[0].to_str().unwrap().to_string();
  info!(" del dir:\n{:?}", &path_to_dir);
  let outdated_books = crud::get_books_contains_path_to_dir(
    path_to_dir, client).await;
  for outdated_book in outdated_books {
    delete_book(outdated_book.path_to_book, client).await;
  }
}

pub async fn rename_dir(old_dir_path: String, new_dir_path: &PathBuf, client: &PrismaClient) {
  let new_dir_path_str = new_dir_path.to_str().unwrap().to_string();

  info!("\n rename dir\n new_path:\n{:?}\nold_path:\n{:?}", &new_dir_path, &old_dir_path);

  for book_item in crud::get_books_contains_path_to_dir(old_dir_path, client).await {
    let new_path_to_book = new_dir_path.join(book_item.file_name).to_str().unwrap().to_string();
    crud::change_path_and_dir(new_path_to_book, book_item.path_to_book, new_dir_path_str.clone(), client).await;
  }
}
