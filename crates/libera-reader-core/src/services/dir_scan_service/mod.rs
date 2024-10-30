use crate::db::models::BookItem;
use crate::utils::{BookPath, BookSize, Hash};
use gxhash::{HashMap, HashMapExt, HashSet, HashSetExt};
use measure_time_macro::measure_time;
use std::ops::Sub;
use tokio::sync::RwLock;
use tracing::{event, Level};
use walkdir::WalkDir;

mod book_adder;


enum BooksLocation {
  Disk,
  DB,
  Both,
  None,
}

pub(crate) async fn run(path_to_scan: &String,
                        book_sizes: &mut RwLock<HashSet<BookSize>>,
                        book_hashes: &mut RwLock<HashSet<Hash>>) {
  let start = std::time::Instant::now();
  let db_book_items = BookItem::get_all();
  let disk_book_items: HashMap<BookPath, BookItem> = get_book_items_from_disk(&path_to_scan);

  event!(Level::INFO, "book items count from db: {:?}", db_book_items.len());

  let db_book_count = db_book_items.len();
  let disk_book_count = disk_book_items.len();

  let (new_book_items, outdated_book_items) = get_new_and_outdated_book_items(disk_book_items, db_book_items);

  event!(Level::INFO, "new books count: {:?}", new_book_items.len());
  event!(Level::INFO, "outdated books count: {:?}", outdated_book_items.len());

  match get_books_location(db_book_count, disk_book_count) {
    BooksLocation::Disk => {
      book_adder::run(new_book_items, book_sizes, book_hashes).await;
    }
    BooksLocation::DB => {
      book_adder::run(new_book_items, book_sizes, book_hashes).await;
    }
    BooksLocation::Both => {
      if new_book_items.len() > 0 {
        book_adder::run(new_book_items, book_sizes, book_hashes).await;
      }
      if outdated_book_items.len() > 0 {
        todo!("make deleting outdated_book_items")
      }
    }
    BooksLocation::None => {}
  };
  event!(Level::INFO, "Dir scan service execution time is: {:?}", start.elapsed());
}

#[measure_time]
fn get_book_items_from_disk(path_to_scan: &String) -> HashMap<BookPath, BookItem> {
  let mut books_from_disk: HashMap<BookPath, BookItem> = HashMap::new();
  for entry in WalkDir::new(path_to_scan) {
    let entry = entry.unwrap();
    if entry.file_type().is_file() {
      let path = entry.path();
      match path.extension() {
        Some(res) => {
          let file_ext = res.to_str().unwrap();
          if ["pdf"].contains(&file_ext) {
            let book_item = BookItem::from_pathbuf(&path.to_path_buf());
            books_from_disk.insert(book_item.path_to_book.clone(), book_item);
          }
        }
        None => {}
      };
    }
  };
  books_from_disk
}

#[measure_time]
fn get_books_location(db_book_count: usize, disk_book_count: usize) -> BooksLocation {
  if db_book_count > 0 && disk_book_count == 0 {
    BooksLocation::DB
  } else if db_book_count == 0 && disk_book_count > 0 {
    BooksLocation::Disk
  } else if db_book_count > 0 && disk_book_count > 0 {
    BooksLocation::Both
  } else {
    BooksLocation::None
  }
}

#[measure_time]
fn get_new_and_outdated_book_items(mut disk_book_items: HashMap<BookPath, BookItem>,
                                   mut db_book_items: HashMap<BookPath, BookItem>) -> (Vec<BookItem>, Vec<BookItem>) {
  let mut disk_book_items_set: HashSet<BookPath> = HashSet::new();
  let mut db_book_items_set: HashSet<BookPath> = HashSet::new();

  let books_paths_from_disk: Vec<BookPath> = disk_book_items.keys().into_iter()
    .map(|i| i.clone()).collect();
  disk_book_items_set.extend(books_paths_from_disk);
  let books_paths_from_db: Vec<BookPath> = db_book_items.keys().into_iter()
    .map(|i| i.clone()).collect();
  db_book_items_set.extend(books_paths_from_db);

  let new_book_items_set = disk_book_items_set.sub(&db_book_items_set);
  let outdated_book_items_set = db_book_items_set.sub(&disk_book_items_set);

  let mut new_book_items: Vec<BookItem> = vec![];
  let mut outdated_book_items: Vec<BookItem> = vec![];
  for i in new_book_items_set {
    new_book_items.push(disk_book_items.remove(&i).unwrap());
  }
  for i in outdated_book_items_set {
    outdated_book_items.push(db_book_items.remove(&i).unwrap());
  }
  (new_book_items, outdated_book_items)
}

