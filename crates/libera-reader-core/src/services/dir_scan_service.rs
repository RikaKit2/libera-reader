use std::collections::HashSet;
use std::ops::Sub;
use std::sync::Arc;
use std::time::Instant;

use tokio::sync::RwLock;
use tracing::info;
use walkdir::WalkDir;

use crate::db::crud;
use crate::db::model::PrismaClient;
use crate::utils::BookData;

async fn get_book_data_by_disk(path_to_scan: &String,
                               target_ext: &Arc<RwLock<HashSet<String>>>) ->
                               (HashSet<BookData>, Vec<String>) {
  let t1 = Instant::now();
  let mut books_data: HashSet<BookData> = HashSet::new();
  let mut books_paths: Vec<String> = vec![];
  for entry in WalkDir::new(path_to_scan) {
    let entry = entry.unwrap();
    if entry.file_type().is_file() {
      let path = entry.path();
      let file_ext = path.extension().unwrap().to_str().unwrap().to_string();
      if target_ext.read().await.contains(&file_ext) {
        let book_data = BookData::from_pathbuf(&path.to_path_buf());
        books_data.insert(book_data);
        books_paths.push(path.to_str().unwrap().to_string());
      }
    }
  };
  info!("\n Duration of receiving books from disk, eq: {:?}, \n num of books from disk eq: {:?}",
    t1.elapsed(), books_data.len());
  (books_data, books_paths)
}

async fn get_book_data_by_db(client: &PrismaClient) -> HashSet<BookData> {
  let t1 = Instant::now();

  let mut books_from_db: HashSet<BookData> = HashSet::new();
  for i in crud::get_books_paths_from_db(client).await {
    books_from_db.insert(BookData::from_book_item(i));
  };

  info!("\n Duration of receiving books from db, eq: {:?}, \n num of books from db eq: {:?}",
   t1.elapsed(), books_from_db.len());
  books_from_db
}

pub async fn run(path_to_scan: &String, client: &PrismaClient, target_ext: &Arc<RwLock<HashSet<String>>>) {
  let start = Instant::now();

  let (book_data_by_disk, books_paths_by_disk) =
    get_book_data_by_disk(path_to_scan, target_ext).await;
  let book_data_by_db = get_book_data_by_db(client).await;

  crud::del_outdated_books(books_paths_by_disk.clone(), client).await;
  crud::mark_paths_to_outdated_user_books_as_invalid(books_paths_by_disk, client).await;

  let new_books_for_db = book_data_by_disk.sub(&book_data_by_db);
  crud::push_new_books_to_db(new_books_for_db, client).await;

  info!("\n Total dir scanning service time, eq: {:?}", start.elapsed());
}

