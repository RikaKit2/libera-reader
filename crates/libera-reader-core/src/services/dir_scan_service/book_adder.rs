use crate::db::crud;
use crate::db::models::BookDataType::{RepeatingSizeBook, UniqueSizeBook};
use crate::db::models::{Book, RepeatSizeBookData, UniqueSizeBookData};
use crate::utils::{calc_file_hash, calc_file_size_in_mb, BookHash, BookPath, BookSize};
use gxhash::{HashMap, HashMapExt};
use itertools::Itertools;
use measure_time_macro::measure_time;
use tracing::info;


struct RepeatingSizeBooks {
  book_size: BookSize,
  books: Vec<Book>,
}

type BooksSameSize = HashMap<BookSize, Vec<Book>>;

pub(crate) async fn run(new_books: Vec<Book>) {
  let start = std::time::Instant::now();
  if new_books.len() > 0 {
    let mut list_repeating_size_books: Vec<RepeatingSizeBooks> = vec![];
    let (unique_size_books, unique_size_book_data) =
      get_unique_size_books(get_list_of_book_sizes(new_books), &mut list_repeating_size_books);

    info!("Len of unique size books: {:?}", unique_size_books.len());
    crud::insert_batch::<Book>(unique_size_books);
    crud::insert_batch::<UniqueSizeBookData>(unique_size_book_data);

    let (repeating_size_books, repeating_size_book_data) =
      get_repeating_size_books(list_repeating_size_books);

    info!("Len of repeating size books: {:?}", repeating_size_books.len());
    crud::insert_batch::<Book>(repeating_size_books);
    crud::insert_batch::<RepeatSizeBookData>(repeating_size_book_data);
  }
  info!("Time to add new books is: {:?}", start.elapsed());
}

//noinspection RsCompileErrorMacro
#[measure_time]
fn get_list_of_book_sizes(new_books: Vec<Book>) -> BooksSameSize {
  let mut list_of_book_sizes: HashMap<BookSize, Vec<Book>> = HashMap::new();

  for mut new_book in new_books {
    let book_size = calc_file_size_in_mb(&new_book.path_to_book).to_string();
    match list_of_book_sizes.get_mut(&book_size) {
      None => {
        new_book.book_data_primary_key = Some(UniqueSizeBook(book_size.clone()));
        list_of_book_sizes.insert(book_size, vec![new_book]);
      }
      Some(group_of_same_size_books) => {
        group_of_same_size_books.into_iter()
          .for_each(|i| i.book_data_primary_key = Some(RepeatingSizeBook(None)));
        new_book.book_data_primary_key = Some(RepeatingSizeBook(None));
        group_of_same_size_books.push(new_book);
      }
    }
  }
  list_of_book_sizes
}

//noinspection RsUnresolvedPath
#[measure_time]
fn get_unique_size_books(list_of_book_sizes: BooksSameSize,
                         list_repeating_size_books: &mut Vec<RepeatingSizeBooks>) -> (Vec<Book>, Vec<UniqueSizeBookData>) {
  let mut unique_size_books: Vec<Book> = vec![];
  let mut unique_size_book_data: Vec<UniqueSizeBookData> = vec![];
  for (book_size, books) in list_of_book_sizes {
    if books.len() >= 1 {
      if books.len() == 1 {
        let book_primary_keys = books.iter().map(|i| i.path_to_book.clone()).collect_vec();
        unique_size_books.extend(books);
        unique_size_book_data.push(UniqueSizeBookData::new(book_size, book_primary_keys));
      } else {
        list_repeating_size_books.push(RepeatingSizeBooks { book_size, books });
      }
    }
  }
  (unique_size_books, unique_size_book_data)
}

//noinspection DuplicatedCode
#[measure_time]
fn get_repeating_size_books(list_repeating_size_books: Vec<RepeatingSizeBooks>) -> (Vec<Book>, Vec<RepeatSizeBookData>) {
  let mut list_of_same_hash_books: HashMap<BookHash, (BookSize, Vec<BookPath>)> = HashMap::new();

  let mut repeating_size_books: Vec<Book> = vec![];
  let mut repeating_size_book_data: Vec<RepeatSizeBookData> = vec![];

  for i in list_repeating_size_books {
    for mut book in i.books {
      let book_hash = calc_file_hash(&book.path_to_book);
      book.book_data_primary_key = Some(RepeatingSizeBook(Some(book_hash.clone())));
      match list_of_same_hash_books.get_mut(&book_hash) {
        None => {
          list_of_same_hash_books.insert(book_hash, (i.book_size.clone(), vec![book.path_to_book.clone()]));
        }
        Some((_, group_of_same_hash_books)) => {
          group_of_same_hash_books.push(book.path_to_book.clone());
        }
      }
      repeating_size_books.push(book);
    }
  }
  for (book_hash, (book_size, books)) in list_of_same_hash_books {
    repeating_size_book_data.push(RepeatSizeBookData::new(book_hash, book_size.clone(), books));
  };
  (repeating_size_books, repeating_size_book_data)
}
