use crate::db::models::{Book, BookDataType, DataOfHashedBook, DataOfUnhashedBook};
use crate::db::models_impl::GetBookData;
use crate::db::{crud, DB};
use crate::services::NOT_CACHED_BOOKS;
use crate::settings::TargetExtensions;
use crate::utils::{calc_file_hash, calc_file_size_in_mb, BookPath, NotCachedBook};
use itertools::Itertools;
use measure_time_macro::measure_time;
use native_db::ToInput;
use std::path::PathBuf;
use std::time::Duration;
use crossbeam_channel::Receiver;
use notify::{Event, EventKind};
use notify::event::{CreateKind, ModifyKind, RemoveKind, RenameMode};
use tracing::Level;
use tracing::{event, info};

#[measure_time]
async fn on_add_new_book(mut new_book: Book, target_ext: &TargetExtensions) {
  if target_ext.read().await.contains(&new_book.ext) {
    info!("\n create new book:\n{:?}", &new_book.path_to_book);
    let book_size = calc_file_size_in_mb(&new_book.path_to_book).to_string();
    match crud::get_primary::<DataOfUnhashedBook>(book_size.clone()) {
      None => {
        let data_type = BookDataType::UniqueSize(book_size.clone());

        NOT_CACHED_BOOKS.write().await.push_front(
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

        new_book.book_data_pk = Some(BookDataType::RepeatingSize(Some(book_hash.clone())));
        crud::update(book_from_db, new_book).unwrap();

        let new_book_data =
          crud::remove::<DataOfUnhashedBook>(unique_book_data).unwrap().to_repeat_size_book_data(book_hash);
        crud::insert::<DataOfHashedBook>(new_book_data).unwrap();
      }
    };
  }
}
#[measure_time]
async fn on_update_path_to_book(old_path: &PathBuf, new_path: &PathBuf, target_ext: &TargetExtensions) {
  let old_book = Book::from_pathbuf(old_path);
  let mut new_book = Book::from_pathbuf(new_path);
  match crud::get_primary::<Book>(old_book.path_to_book.clone()) {
    None => {
      on_add_new_book(new_book, target_ext).await;
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
async fn on_rename_dir(old_dir_path: String, new_dir_path: &PathBuf) {
  info!("\n rename dir\n new_path:\n{:?}\nold_path:\n{:?}", &new_dir_path, &old_dir_path);

  for old_book in get_books_located_in_dir(old_dir_path) {
    let mut new_book = old_book.clone();
    new_book.dir_name = new_dir_path.file_name().unwrap().to_str().unwrap().to_string();
    new_book.path_to_dir = new_dir_path.to_str().unwrap().to_string();
    crud::update(old_book, new_book).unwrap();
  }
}
fn get_books_located_in_dir(path_to_dir: String) -> Vec<Book> {
  let r_conn = DB.r_transaction().unwrap();
  let books: Vec<Book> = r_conn.scan().primary().unwrap().start_with(path_to_dir)
    .unwrap().try_collect().unwrap();
  books
}
#[measure_time]
fn on_delete_dir(path_to_dir: String) {
  info!("\n del dir:\n{:?}", &path_to_dir);
  for old_book in get_books_located_in_dir(path_to_dir) {
    on_delete_book(old_book);
  }
}
#[measure_time]
fn on_delete_book(old_book: Book) {
  match old_book.book_data_pk.unwrap() {
    BookDataType::UniqueSize(book_size) => {
      let book_data = crud::get_primary::<DataOfUnhashedBook>(book_size).unwrap();
      del_event_handler(book_data, &old_book.path_to_book);
    }
    BookDataType::RepeatingSize(book_hash) => {
      let book_data = crud::get_primary::<DataOfHashedBook>(book_hash.unwrap()).unwrap();
      del_event_handler(book_data, &old_book.path_to_book);
    }
  }
}
fn del_event_handler<T: ToInput + GetBookData>(data: T, path_to_book: &BookPath) {
  let rw_conn = DB.rw_transaction().unwrap();
  let book_data = data.get_book_data_as_ref();
  if book_data.favorite == false &&
    book_data.in_history == false {
    if book_data.books_pk.len() == 0 {
      let book_data = data.get_book_data();
      mark_book_paths_as_invalid(book_data.books_pk);
    } else {
      rw_conn.remove(data).unwrap();
      info!("\n del book:\n{:?}", path_to_book);
    }
  } else {
    let book_data = data.get_book_data_as_ref();
    for book_path in book_data.books_pk.clone() {
      let book = crud::get_primary::<Book>(book_path).unwrap();
      crud::remove::<Book>(book).unwrap();
    }
    rw_conn.remove(data).unwrap();
    info!("\n del book:\n{:?}", path_to_book);
  }
  rw_conn.commit().unwrap();
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
async fn event_processing(event: Event, target_ext: &TargetExtensions) {
  match event {
    Event { kind, paths, attrs: _attrs } => {
      match kind {
        EventKind::Create(create_kind) => {
          match create_kind {
            CreateKind::File => {
              let new_book = Book::from_pathbuf(&paths[0]);
              on_add_new_book(new_book, target_ext).await;
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
                  if new_path.is_dir() {
                    let old_dir_path = old_path.to_str().unwrap().to_string();
                    on_rename_dir(old_dir_path, new_path).await;
                  } else {
                    on_update_path_to_book(old_path, new_path, target_ext).await;
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
              on_delete_book(Book::from_pathbuf(&paths[0]));
            }
            RemoveKind::Folder => {
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

pub async fn run(notify_events: Receiver<notify::Result<Event>>, target_ext: TargetExtensions) {
  loop {
    match notify_events.try_recv() {
      Ok(res) => {
        match res {
          Ok(event) => {
            event_processing(event, &target_ext).await;
          }
          Err(_) => {}
        }
      }
      Err(_) => {}
    }
    tokio::time::sleep(Duration::from_secs(1)).await;
  }
}
