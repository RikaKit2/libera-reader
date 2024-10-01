use crate::db::models::{BookData, BookItem, BookMark, Settings};
use crate::utils::calc_file_size_in_mb;
use gxhash::{HashMap, HashMapExt, HashSet, HashSetExt};
use itertools::Itertools;
use native_db::db_type::KeyDefinition;
use native_db::transaction::{RTransaction, RwTransaction};
use native_db::*;
use once_cell::sync::Lazy;
use queues::*;
use std::ops::Sub;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use walkdir::WalkDir;


static MODELS: Lazy<Models> = Lazy::new(|| {
  let mut models = Models::new();
  models.define::<Settings>().unwrap();
  models.define::<BookMark>().unwrap();
  models.define::<BookData>().unwrap();
  models.define::<BookItem>().unwrap();
  models
});

fn get_book_data_from_disk(path_to_scan: &String) -> HashSet<BookItem> {
  let t1 = Instant::now();
  let mut books_from_disk: HashSet<BookItem> = HashSet::new();
  for entry in WalkDir::new(path_to_scan) {
    let entry = entry.unwrap();
    if entry.file_type().is_file() {
      let path = entry.path();
      match path.extension() {
        Some(res) => {
          let file_ext = res.to_str().unwrap();
          if ["pdf"].contains(&file_ext) {
            books_from_disk.insert(BookItem::from_pathbuf(&path.to_path_buf()));
          }
        }
        None => {}
      };
    }
  };
  println!("\n Duration of receiving books from disk, eq: {:?}, \n num of books from disk eq: {:?}",
           t1.elapsed(), books_from_disk.len());
  books_from_disk
}

fn del_outdated_books(rw_conn: &RwTransaction, r_conn: &RTransaction, outdated_books: HashSet<BookItem>) {
  if outdated_books.len() > 0 {
    for old_book_item in outdated_books {
      let book_data_id = old_book_item.book_data_link.clone().unwrap().as_bytes().to_vec();
      let book_data: BookData = r_conn.get().primary(book_data_id).unwrap().unwrap();

      if book_data.in_history == true || book_data.favorite == true {
        let mut new_book = old_book_item.clone();
        new_book.path_is_valid = false;
        rw_conn.update(old_book_item, new_book).unwrap();
      } else {
        rw_conn.remove(old_book_item).unwrap();
      }
    }
  }
}

fn calc_size_of_books(new_book_items: HashSet<BookItem>) -> (Vec<(Vec<BookItem>, String)>, Vec<(Vec<BookItem>, String)>) {
  let mut books_of_known_size: HashMap<String, Vec<BookItem>> = HashMap::new();

  for book_item in new_book_items {
    let book_size = calc_file_size_in_mb(&book_item.path_to_book.clone()).to_string();
    let repeating_size_books = books_of_known_size.get_mut(&book_size);

    if repeating_size_books.is_none() {
      books_of_known_size.insert(book_size, vec![book_item]).unwrap();
    } else {
      repeating_size_books.unwrap().push(book_item);
    }
  }

  let mut repeat_size_books: Vec<(Vec<BookItem>, String)> = vec![];
  let mut unique_size_books: Vec<(Vec<BookItem>, String)> = vec![];

  for (book_size, book_items) in books_of_known_size {
    if book_items.len() > 1 {
      repeat_size_books.push((book_items, book_size));
    } else {
      unique_size_books.push((book_items, book_size));
    }
  }
  (repeat_size_books, unique_size_books)
}

fn add_book_data_to_db(rw_conn: &RwTransaction, hash: String, file_size: String) -> BookData {
  let book_data = BookData {
    hash,
    file_size,
    cached: false,
    title: None,
    author: None,
    page_count: None,
    in_history: false,
    favorite: false,
    last_page_number: 0,
    latest_opening_in: None,
  };
  rw_conn.insert(book_data.clone()).expect("Failed to insert book_data into the db.");
  book_data
}

fn insert_book_item_with_book_data(rw_conn: &RwTransaction, book_data: BookData, book_items: &mut Vec<BookItem>) {
  for book_item in book_items {
    book_item.book_data_link = Some(book_data.hash.clone());
    rw_conn.insert(book_item.clone()).unwrap();
  }
}

fn insert_book_items_to_db(rw_conn: &RwTransaction, new_book_items: Vec<BookItem>) {
  for book_item in new_book_items {
    rw_conn.insert(book_item).unwrap();
  }
}

fn add_new_books_to_db(rw_conn: &RwTransaction, unique_size_books: &mut Vec<(Vec<BookItem>, String)>) {
  for (book_items, book_size) in unique_size_books {
    let book_data: Option<BookData> = rw_conn.get().secondary(
      KeyDefinition::new(1, 1, "compute_secondary_key", Default::default()),
      book_size.clone().as_bytes().to_vec()).unwrap();
    match book_data {
      None => {}
      Some(book_data) => {
        insert_book_item_with_book_data(rw_conn, book_data, book_items);
      }
    }
  }
}

fn calc_book_hashes(books_with_repeating_size: Vec<BookItem>) {}

pub fn run() {
  let path_to_scan = "".to_string();
  let db = Builder::new().create_in_memory(&MODELS).unwrap();
  let r_conn = db.r_transaction().unwrap();
  let rw_conn = db.rw_transaction().unwrap();

  let disk_book_items = get_book_data_from_disk(&path_to_scan);

  let db_book_items: Vec<BookItem> = r_conn.scan().primary().unwrap().all().try_collect().unwrap();
  let db_book_items: HashSet<BookItem> = db_book_items.into_iter().filter(|i| i.path_is_valid).collect();

  let outdated_book_items = db_book_items.sub(&disk_book_items);
  let new_book_items = disk_book_items.sub(&db_book_items);

  del_outdated_books(&rw_conn, &r_conn, outdated_book_items);

  let books_for_mupdf_service: Arc<Mutex<Queue<String>>> = Arc::from(Mutex::new(Queue::new()));

  if new_book_items.len() > 0 {
    let (repeat_size_books, unique_size_books) = calc_size_of_books(new_book_items);
  }

  rw_conn.commit().unwrap();
}

