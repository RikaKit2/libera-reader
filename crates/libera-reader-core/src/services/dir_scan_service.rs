use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use tokio::sync::RwLock;
use walkdir::WalkDir;

use crate::book_manager::BookManager;
use crate::db::crud;
use crate::db::model::PrismaClient;

async fn get_books_paths_from_disk(path_to_scan: &String, target_ext: &Arc<RwLock<HashSet<String>>>) -> HashSet<PathBuf> {
  let mut books_paths_from_disk: HashSet<PathBuf> = HashSet::new();
  for entry in WalkDir::new(path_to_scan) {
    let entry = entry.unwrap();
    if entry.file_type().is_file() {
      let path = entry.path();
      let file_ext = path.extension().unwrap().to_str().unwrap().to_string();
      if target_ext.read().await.contains(&file_ext) {
        books_paths_from_disk.insert(path.to_path_buf());
      }
    }
  };
  books_paths_from_disk
}

async fn get_books_paths_from_db(client: &PrismaClient) -> HashSet<PathBuf> {
  let mut books_from_db: HashSet<PathBuf> = HashSet::new();
  for i in crud::get_books_paths_from_db(client).await {
    let book_item = PathBuf::from(i.path_to_book);
    books_from_db.insert(book_item);
  };
  books_from_db
}

pub async fn run(path_to_scan: &String, book_manager: Arc<RwLock<BookManager>>) {
  let start = Instant::now();

  let t1 = Instant::now();
  let books_paths_from_disk = get_books_paths_from_disk(path_to_scan,
                                                        &book_manager.read().await.target_ext).await;
  tracing::info!("\n Duration of receiving books from disk, eq: {:?}, \nnum of books from disk eq: {:?}",
    t1.elapsed(), books_paths_from_disk.len());

  let t1 = Instant::now();
  let books_from_db = get_books_paths_from_db(&book_manager.write().await.client).await;
  tracing::info!("\n Duration of receiving books from db, eq: {:?}, \nnum of books from db eq: {:?}",
   t1.elapsed(), books_from_db.len());

  let t1 = Instant::now();
  let new_books_for_db = books_paths_from_disk.difference(&books_from_db);
  tracing::info!("\n Duration of calc of new books, eq: {:?}", t1.elapsed());

  let t1 = Instant::now();
  let outdated_books = books_from_db.difference(&books_paths_from_disk);
  tracing::info!("\n Duration of calc of outdated books, eq: {:?}", t1.elapsed());

  let t1 = Instant::now();
  for i in new_books_for_db {
    book_manager.write().await.add_new_book(i).await;
  }
  tracing::info!("\n Duration of adding new documents, eq: {:?}", t1.elapsed());

  let t1 = Instant::now();
  for outdated_book in outdated_books {
    book_manager.write().await.delete_book(outdated_book.to_str().unwrap().to_string()).await;
  }
  tracing::info!("\n Duration of deletion outdated books, eq: {:?}", t1.elapsed());

  tracing::info!("\n total dir scanning service time, eq: {:?}", start.elapsed());
}

