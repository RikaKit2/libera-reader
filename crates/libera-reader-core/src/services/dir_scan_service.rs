use gxhash::HashSet;
use gxhash::HashSetExt;
use rayon::prelude::*;
use std::iter::zip;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::info;
use walkdir::WalkDir;

use crate::db::crud;
use crate::db::model::PrismaClient;
use crate::utils::{calc_file_hash, calc_file_size_in_mb, BookData, BookHashAndSize};

async fn get_book_data_from_disk(path_to_scan: &String, target_ext: &Arc<RwLock<HashSet<String>>>) -> (Vec<BookData>, Vec<String>) {
  let t1 = Instant::now();
  let mut books_data: Vec<BookData> = vec![];
  let mut books_paths: Vec<String> = vec![];
  for entry in WalkDir::new(path_to_scan) {
    let entry = entry.unwrap();
    if entry.file_type().is_file() {
      let path = entry.path();
      match path.extension() {
        Some(res) => {
          let file_ext = res.to_str().unwrap().to_string();
          if target_ext.read().await.contains(&file_ext) {
            let book_data = BookData::from_pathbuf(&path.to_path_buf());
            books_data.push(book_data);
            books_paths.push(path.to_str().unwrap().to_string());
          }
        }
        None => {}
      };
    }
  };
  info!("\n Duration of receiving books from disk, eq: {:?}, \n num of books from disk eq: {:?}",
    t1.elapsed(), books_paths.len());
  (books_data, books_paths)
}

async fn get_book_data_from_db(client: &PrismaClient) -> HashSet<String> {
  let t1 = Instant::now();
  let mut books_from_db: HashSet<String> = gxhash::HashSet::new();
  for i in crud::get_books_paths_from_db(client).await {
    books_from_db.insert(i.path_to_book);
  };

  info!("\n Duration of receiving books from db, eq: {:?}, \n num of books from db eq: {:?}",
   t1.elapsed(), books_from_db.len());
  books_from_db
}

fn get_new_books_for_db(book_data_from_disk: Vec<BookData>, books_paths_from_db: &HashSet<String>) -> Vec<BookData> {
  let mut new_books_for_db = vec![];
  for i in book_data_from_disk {
    if !books_paths_from_db.contains(&i.path_to_book) {
      new_books_for_db.push(i);
    }
  };
  new_books_for_db
}

async fn add_new_books_to_db(new_books: Vec<BookData>, client: Arc<PrismaClient>) {
  let t1 = Instant::now();
  let data: Vec<BookHashAndSize> = new_books.par_iter().map(move |i| {
    let book_hash = calc_file_hash(&i.path_to_book);
    let book_size = calc_file_size_in_mb(&i.path_to_book);
    BookHashAndSize { book_hash, book_size }
  }).collect();
  info!("\n Duration of calc book hashes: {:?}", t1.elapsed());

  let t2 = Instant::now();
  for (i, k) in zip(new_books, data) {
    let data_id = crud::get_book_data_id_upsert(k.book_hash, k.book_size, &client).await;
    crud::create_book_item(i, data_id, &client).await;
  }
  info!("\n Duration getting book_data_id: {:?}", t2.elapsed());
}

pub async fn run(path_to_scan: &String, client: Arc<PrismaClient>, target_ext: &Arc<RwLock<HashSet<String>>>) {
  let start = Instant::now();

  let (book_data_from_disk, books_paths_by_disk) =
    get_book_data_from_disk(path_to_scan, target_ext).await;
  let books_paths_from_db = get_book_data_from_db(&client).await;
  if books_paths_from_db.len() > 0 {
    crud::del_outdated_books(books_paths_by_disk.clone(), &client).await;
    crud::mark_paths_to_outdated_user_books_as_invalid(books_paths_by_disk, &client).await;
  }

  let t1 = Instant::now();
  let new_books_for_db = get_new_books_for_db(book_data_from_disk, &books_paths_from_db);

  // add_new_books_to_db(new_books_for_db, client).await;
  info!("\n Duration creation new books eq: {:?}", t1.elapsed());

  info!("\n Total dir scanning service time, eq: {:?}", start.elapsed());
}

