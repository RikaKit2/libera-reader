use crate::db::models::{Book, BookDataType, DataOfHashedBook, DataOfUnhashedBook};
use crate::db::models_impl::GetBookData;
use crate::db::{crud, DB};
use crate::types::{BookPath, NotCachedBook};
use crate::utils::{calc_file_hash, calc_file_size_in_mb};
use crate::vars::{NOT_CACHED_BOOKS, SETTINGS};
use itertools::Itertools;
use measure_time_macro::measure_time;
use native_db::ToInput;
use notify::event::{CreateKind, ModifyKind, RemoveKind, RenameMode};
use notify::{Event, EventKind};
use std::path::PathBuf;
use tracing::{error, info};


#[measure_time]
fn on_add_new_book(mut new_book: Book) {
  if SETTINGS.read().unwrap().target_ext.contains(&new_book.ext) {
    info!("\n create new book:\n{:?}", &new_book.path_to_book);
    let book_size = calc_file_size_in_mb(&new_book.path_to_book).to_string();
    match crud::get_primary::<DataOfUnhashedBook>(book_size.clone()) {
      None => {
        let data_type = BookDataType::UniqueSize(book_size.clone());

        NOT_CACHED_BOOKS.write().unwrap().push_front(
          NotCachedBook { data_type: data_type.clone(), path_to_book: new_book.path_to_book.clone() });

        crud::insert::<DataOfUnhashedBook>(
          DataOfUnhashedBook::new(book_size, vec![new_book.path_to_book.clone()])).unwrap();
        new_book.book_data_pk = Some(data_type);
        crud::insert::<Book>(new_book).unwrap();
      }
      Some(unique_book_data) => {
        let book_hash = match &unique_book_data.book_hash {
          None => {
            calc_file_hash(&new_book.path_to_book)
          }
          Some(book_hash) => {
            book_hash.clone()
          }
        };
        let book_path = &unique_book_data.book_data.books_pk[0];
        let book_from_db = crud::get_primary::<Book>(book_path.clone()).unwrap();

        new_book.book_data_pk = Some(BookDataType::RepeatingSize(book_hash.clone()));
        crud::update(book_from_db, new_book).unwrap();

        let new_book_data =
          crud::remove::<DataOfUnhashedBook>(unique_book_data).unwrap().to_repeat_size_book_data(book_hash);
        crud::insert::<DataOfHashedBook>(new_book_data).unwrap();
      }
    };
  }
}
#[measure_time]
fn on_update_path_to_book(old_path: &PathBuf, new_path: &PathBuf) {
  let old_book = Book::from_pathbuf(old_path);
  let mut new_book = Book::from_pathbuf(new_path);
  match crud::get_primary::<Book>(old_book.path_to_book.clone()) {
    None => {
      on_add_new_book(new_book);
    }
    Some(book_from_db) => {
      info!("\n rename book data\n new_path:\n{:?}\n old_path:\n{:?}", &new_book.path_to_book,
          &book_from_db.path_to_book);

      new_book.book_data_pk = book_from_db.book_data_pk.clone();
      crud::update(new_book, old_book).unwrap();
    }
  }
}
#[measure_time]
fn on_rename_dir(old_dir_path: String, new_dir_path: &PathBuf) {
  info!("\n rename dir\n new_path:\n{:?}\nold_path:\n{:?}", &new_dir_path, &old_dir_path);

  for old_book in get_books_located_in_dir(old_dir_path) {
    let mut new_book = old_book.clone();
    new_book.dir_name = new_dir_path.file_name().unwrap().to_str().unwrap().to_string();
    new_book.path_to_dir = new_dir_path.to_str().unwrap().to_string();
    crud::update(old_book, new_book).unwrap();
  }
}
#[measure_time]
fn on_delete_dir(path_to_dir: String) {
  info!("\n del dir:\n{:?}", &path_to_dir);
  for old_book in get_books_located_in_dir(path_to_dir) {
    on_delete_book(old_book.path_to_book.as_str());
  }
}
#[measure_time]
fn on_delete_book(path_to_book: &str) {
  info!("start del");
  match crud::get_primary::<Book>(path_to_book) {
    None => {
      info!("book not found: {path_to_book}", )
    }
    Some(old_book) => {
      match old_book.book_data_pk.clone() {
        None => {
          match crud::remove::<Book>(old_book) {
            Ok(_) => {
              info!("the book on this path was deleted: \n{}", path_to_book);
            }
            Err(e) => {
              error!("{:?}", e)
            }
          };
        }
        Some(data_type) => {
          match data_type {
            BookDataType::UniqueSize(book_size) => {
              let book_data = crud::get_primary::<DataOfUnhashedBook>(book_size).unwrap();
              remove_book_and_book_data(book_data, old_book);
            }
            BookDataType::RepeatingSize(book_hash) => {
              let book_data = crud::get_primary::<DataOfHashedBook>(book_hash).unwrap();
              remove_book_and_book_data(book_data, old_book);
            }
          };
        }
      };
    }
  };
}

fn get_books_located_in_dir(path_to_dir: String) -> Vec<Book> {
  let r_conn = DB.r_transaction().unwrap();
  let books: Vec<Book> = r_conn.scan().primary().unwrap().start_with(path_to_dir)
    .unwrap().try_collect().unwrap();
  books
}
fn remove_book_and_book_data<T: ToInput + GetBookData>(data: T, book: Book) {
  let rw_conn = DB.rw_transaction().unwrap();
  let book_data = data.get_book_data_as_ref();
  if book_data.favorite == false && book_data.in_history == false {
    if book_data.books_pk.len() == 1 {
      crud::remove::<Book>(book).unwrap();
      crud::remove::<T>(data).unwrap();
    } else if book_data.books_pk.len() > 1 {
      for i in book_data.books_pk.clone() {
        let book_for_deletion = crud::get_primary::<Book>(i).unwrap();
        crud::remove::<Book>(book_for_deletion).unwrap();
      }
      crud::remove::<T>(data).unwrap();
    }
  } else {
    mark_book_paths_as_invalid(book_data.books_pk.clone());
  }
  rw_conn.commit().unwrap();
  info!("remove_book_and_book_data");
}
fn mark_book_paths_as_invalid(books_pk: Vec<BookPath>) {
  books_pk.into_iter().for_each(|book_path| {
    match crud::get_primary::<Book>(book_path) {
      None => {}
      Some(old_book) => {
        let mut new_book = old_book.clone();
        new_book.path_is_valid = false;
        crud::update::<Book>(old_book, new_book).unwrap()
      }
    }
  });
}

fn event_processing(event: Event) {
  match event {
    Event { kind, paths, attrs: _attrs } => {
      match kind {
        EventKind::Create(create_kind) => {
          match create_kind {
            CreateKind::File => {
              let path_to_file = paths[0].to_str().unwrap();
              info!("new file: {path_to_file}");
              let new_book = Book::from_pathbuf(&paths[0]);
              on_add_new_book(new_book);
            }
            _ => {}
          }
        }
        EventKind::Modify(modify_kind) => {
          match modify_kind {
            ModifyKind::Name(rename_mode) => {
              match rename_mode {
                RenameMode::Both => {
                  let old_path = &paths[0];
                  let new_path = &paths[1];
                  let old_path_str = old_path.to_str().unwrap();
                  let new_path_str = new_path.to_str().unwrap();

                  if new_path.is_dir() {
                    info!("rename dir old_path: {old_path_str}\nnew_path: {new_path_str}");
                    let old_dir_path = old_path.to_str().unwrap().to_string();
                    on_rename_dir(old_dir_path, new_path);
                  } else {
                    info!("rename file old_path: {old_path_str}\nnew_path: {new_path_str}");
                    on_update_path_to_book(old_path, new_path);
                  }
                }
                RenameMode::From => {}
                RenameMode::To => {}
                _ => {}
              }
            }
            _ => {}
          }
        }
        EventKind::Remove(remove_kind) => {
          match remove_kind {
            RemoveKind::File => {
              let path_to_file = paths[0].to_str().unwrap();
              info!("this file has been deleted: {path_to_file}");
              on_delete_book(paths[0].to_str().unwrap());
            }
            RemoveKind::Folder => {
              let path_to_dir = &paths[0].to_str().unwrap();
              info!("this folder has been deleted: {path_to_dir}");
              on_delete_dir(paths[0].to_str().unwrap().to_string());
            }
            _ => {}
          }
        }
        _ => {}
      }
    }
  }
}

//noinspection DuplicatedCode
pub async fn run() {
  loop {
    match SETTINGS.write().unwrap().notify_receiver.try_recv() {
      Ok(res) => {
        match res {
          Ok(event) => {
            event_processing(event);
          }
          Err(_) => {}
        }
      }
      Err(_) => {}
    }
  }
}
