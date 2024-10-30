use crate::db::models::{BookData, BookItem, HashedBookData, InsertBatch};
use crate::utils::{calc_file_hash, calc_file_size_in_mb, BookSize, Hash};
use gxhash::{HashMap, HashMapExt, HashSet};
use measure_time_macro::measure_time;
use tokio::sync::RwLock;
use tracing::{event, Level};


pub(crate) async fn run(new_book_items: Vec<BookItem>,
                        book_sizes: &mut RwLock<HashSet<BookSize>>,
                        book_hashes: &mut RwLock<HashSet<Hash>>) {
  let start = std::time::Instant::now();
  let (books_same_size, new_unique_book_data) = get_same_size_books(new_book_items, book_sizes).await;
  let (mut hashed_books, unique_size_books) = get_unique_and_hashed_books(books_same_size);
  let new_hashed_book_data = get_new_repetitive_book_data(&mut hashed_books, book_hashes).await;

  event!(Level::INFO, "len of unique size books: {:?}", unique_size_books.len());
  event!(Level::INFO, "len of hashed books: {:?}", hashed_books.len());

  BookItem::insert_batch(unique_size_books);
  BookItem::insert_batch(hashed_books);

  BookData::insert_batch(new_unique_book_data);
  HashedBookData::insert_batch(new_hashed_book_data);
  event!(Level::INFO, "time to add new books is: {:?}", start.elapsed());
}

#[measure_time]
async fn get_same_size_books(new_book_items: Vec<BookItem>,
                             book_sizes: &mut RwLock<HashSet<BookSize>>) -> (Vec<Vec<BookItem>>, Vec<BookData>) {
  let mut new_unique_book_data: Vec<BookData> = vec![];
  let mut result: HashMap<BookSize, Vec<BookItem>> = HashMap::new();

  for mut new_book in new_book_items {
    let book_size = calc_file_size_in_mb(&new_book.path_to_book).to_string();
    let books_same_size = result.get_mut(&book_size);

    if !book_sizes.read().await.contains(&book_size) {
      new_unique_book_data.push(BookData::new(book_size.clone()));
      book_sizes.write().await.insert(book_size.clone());
    }

    if books_same_size.is_none() {
      new_book.book_data_link = Some(book_size.clone());
      result.insert(book_size, vec![new_book]);
    } else {
      new_book.book_data_link = Some(book_size);
      books_same_size.unwrap().push(new_book);
    }
  }
  let books_same_size: Vec<Vec<BookItem>> = result.into_values().map(|i| i).collect();

  (books_same_size, new_unique_book_data)
}

#[measure_time]
fn get_unique_and_hashed_books(books_same_size: Vec<Vec<BookItem>>) -> (Vec<BookItem>, Vec<BookItem>) {
  let mut repeat_size_books: Vec<BookItem> = vec![];
  let mut unique_size_books: Vec<BookItem> = vec![];

  for mut book_items in books_same_size {
    if book_items.len() > 1 {
      for book_item in book_items.iter_mut() {
        book_item.unique_file_size = Some(false);
      }
      repeat_size_books.extend(book_items);
    } else {
      let mut book_item = book_items.swap_remove(0);
      book_item.unique_file_size = Some(true);
      unique_size_books.push(book_item);
    }
  }
  (repeat_size_books, unique_size_books)
}

#[measure_time]
fn calc_books_hashes(repeat_size_books: &mut Vec<BookItem>) -> Vec<(Hash, BookSize)> {
  let mut result: Vec<(Hash, BookSize)> = vec![];
  for book_item in repeat_size_books {
    let book_hash = calc_file_hash(&book_item.path_to_book);
    let book_size = calc_file_size_in_mb(&book_item.path_to_book).to_string();
    book_item.unique_file_size = Some(false);
    book_item.book_data_link = Some(book_hash.clone());
    result.push((book_hash, book_size))
  }
  result
}

#[measure_time]
async fn get_new_repetitive_book_data(repeat_size_books: &mut Vec<BookItem>,
                                      hashed_book_data: &mut RwLock<HashSet<Hash>>) -> Vec<HashedBookData> {
  let mut new_repetitive_book_data: Vec<HashedBookData> = vec![];
  let hashed_books = calc_books_hashes(repeat_size_books);

  for (book_hash, book_size) in hashed_books {
    if !hashed_book_data.read().await.contains(&book_hash) {
      new_repetitive_book_data.push(HashedBookData::new(book_hash.clone(), book_size));
      hashed_book_data.write().await.insert(book_hash);
    }
  }
  new_repetitive_book_data
}
