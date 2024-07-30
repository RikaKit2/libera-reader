use std::collections::{HashSet, VecDeque};
use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::RwLock;
use tracing::info;

use crate::db::crud;
use crate::db::model::PrismaClient;
use crate::utils::{BookData, calc_gxhash_of_file, calc_file_size_in_mb, NotCachedBook};

pub struct BookManager {
  pub(crate) target_ext: Arc<RwLock<HashSet<String>>>,
  pub(crate) client: Arc<PrismaClient>,
  not_cached_books: Arc<RwLock<VecDeque<NotCachedBook>>>,
}

impl BookManager {
  pub fn new(target_ext: Arc<RwLock<HashSet<String>>>, client: Arc<PrismaClient>) -> BookManager {
    BookManager {
      target_ext,
      not_cached_books: Arc::from(RwLock::from(VecDeque::new())),
      client,
    }
  }
  pub async fn add_book(&mut self, book: BookData) {
    if self.target_ext.read().await.contains(&book.ext) {
      info!("\n create new book:\n{:?}", &book.path_to_book);
      let book_hash = calc_gxhash_of_file(&book.path_to_book);
      match crud::get_book_data(book_hash.clone(), &self.client).await {
        None => {
          let book_size = calc_file_size_in_mb(&book.path_to_book);
          let book_data = crud::create_book_data(book_hash.clone(), book_size, &self.client).await;
          crud::create_book_item(book.path_to_book.clone(), book_data.unwrap().id, book.path_to_dir,
                                 book.dir_name, book.book_name, book.ext, &self.client).await;
          self.not_cached_books.write().await.push_front(
            NotCachedBook { book_hash, path_to_book: book.path_to_book }
          );
        }
        Some(book_data) => {
          crud::create_book_item(book.path_to_book, book_data.id, book.path_to_dir,
                                 book.dir_name, book.book_name, book.ext, &self.client).await;
        }
      }
    }
  }
  pub async fn rename_data(&mut self, old: BookData, new: BookData) {
    match crud::get_book_item_by_path(old.path_to_book, &self.client).await {
      None => {
        self.add_book(new).await;
      }
      Some(old_book) => {
        info!("\n rename book data\n new_path:\n{:?}\n old_path:\n{:?}", &new.path_to_book,
          &old_book.path_to_book);

        let path_to_dir_eq = &new.path_to_dir == &old.path_to_dir;
        let book_name_eq = &new.book_name == &old.book_name;
        let ext_eq = &new.ext == &old.ext;

        if path_to_dir_eq == false {
          crud::change_path_and_dir(new.path_to_book, old_book.path_to_book, new.path_to_dir, new.dir_name, &self.client).await;
          return;
        } else if book_name_eq == false {
          crud::change_path_and_book_name(new.path_to_book, old_book.path_to_book, new.book_name, &self.client).await;
          return;
        } else if ext_eq == false {
          crud::change_path_and_ext(new.path_to_book, old_book.path_to_book, new.ext, &self.client).await;
        }
      }
    }
  }
  pub async fn delete_book(&mut self, old_path_to_book: String) {
    match crud::get_book_item_by_path(old_path_to_book, &self.client).await {
      None => {}
      Some(book_item) => {
        info!("\n del book:\n{:?}", &book_item.path_to_book);
        crud::del_book(book_item, &self.client).await;
      }
    }
  }
  pub async fn delete_dir(&mut self, paths: Vec<PathBuf>) {
    let path_to_dir = paths[0].to_str().unwrap().to_string();
    info!("\n del dir:\n{:?}", &path_to_dir);
    let outdated_books = crud::get_books_contains_path_to_dir(
      path_to_dir, &self.client).await;
    for outdated_book in outdated_books {
      self.delete_book(outdated_book.path_to_book).await;
    }
  }
  pub async fn rename_dir(&mut self, old_dir_path: String, new_dir_path: &PathBuf) {
    let new_path_to_dir_str = new_dir_path.to_str().unwrap().to_string();
    let new_dir_name = new_dir_path.file_name().unwrap().to_str().unwrap().to_string();
    info!("\n rename dir\n new_path:\n{:?}\nold_path:\n{:?}", &new_dir_path, &old_dir_path);

    for book_item in crud::get_books_contains_path_to_dir(old_dir_path, &self.client).await {
      let new_path_to_book = new_dir_path.join(book_item.book_name).to_str().unwrap().to_string();
      crud::change_path_and_dir(new_path_to_book, book_item.path_to_book,
                                new_path_to_dir_str.clone(), new_dir_name.clone(), &self.client).await;
    }
  }
}
