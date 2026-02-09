use crate::db::models::books::book::{BookPath, BookSize};
use crate::db::models::books::book_hashes::BookHashes;
use crate::db::models::books::{BookType, DuplicateBookData};

use crate::{
  db::{DB, models::books::book::Book},
  types::HashMap,
};
use native_db::*;
#[allow(unused_imports)]
use native_model::{Model, native_model};
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Serialize, Deserialize, Clone)]
#[native_model(id = 2, version = 1)]
#[native_db]
pub struct BookSizes {
  #[primary_key]
  pub book_size: BookSize,
  pub book_type: BookType,
}
impl BookSizes {
  fn new(book_size: BookSize, book_type: BookType) -> Self {
    Self { book_size, book_type }
  }
  fn get(book_size: BookSize, db: &DB) -> anyhow::Result<Option<Self>> {
    db.get_primary::<Self>(book_size)
  }
  pub(crate) async fn insert_book(new_book: &Book, db: &DB) -> anyhow::Result<()> {
    let start_time = std::time::Instant::now();

    match BookSizes::get(new_book.book_size.clone(), db)? {
      Some(old_self) => {
        let mut updated_self = old_self.clone();
        match updated_self.book_type {
          BookType::UniqueSize { book_path, mutool_data } => {
            let mut map = HashMap::default();
            map.insert(book_path, DuplicateBookData::MutoolData(mutool_data));
            map.insert(new_book.book_path.clone(), DuplicateBookData::MutoolData(None));
            updated_self.book_type = BookType::DuplicateSize(map);
            db.update(old_self, updated_self)?;
          }
          BookType::DuplicateSize(mut hash_map) => {
            match hash_map.contains_key(&new_book.book_path) {
              true => {}
              false => {
                hash_map.insert(new_book.book_path.clone(), DuplicateBookData::MutoolData(None));
                let updated_self = BookSizes { book_size: updated_self.book_size, book_type: BookType::DuplicateSize(hash_map) };
                db.update(old_self, updated_self)?;
              }
            };
          }
        };
      }
      None => {
        db.insert::<Self>(Self::new(new_book.book_size.clone(), BookType::UniqueSize { book_path: new_book.book_path.clone(), mutool_data: None }))?;
      }
    };

    let total_time = start_time.elapsed();
    info!("The total time for adding a book in book_sizes: {:?}", &total_time);
    Ok(())
  }
  pub(crate) fn remove_book(book_size: BookSize, target_book_path: &BookPath, db: &DB) -> anyhow::Result<()> {
    match Self::get(book_size, db)? {
      Some(old_self) => {
        let mut updated_self = old_self.clone();
        match &mut updated_self.book_type {
          BookType::UniqueSize { book_path, mutool_data: _ } => {
            match target_book_path.eq(book_path) {
              true => {
                db.remove::<Self>(old_self)?;
              }
              false => {}
            };
          }
          BookType::DuplicateSize(hash_map) => {
            match hash_map.swap_remove(target_book_path) {
              Some(data) => {
                match hash_map.is_empty() {
                  true => {
                    match data {
                      DuplicateBookData::BookHash(book_hash) => {
                        BookHashes::remove_book(book_hash, target_book_path, db)?;
                      }
                      DuplicateBookData::MutoolData(_mutool_data) => {}
                    };
                    db.remove::<Self>(old_self)?;
                  }
                  false => {
                    db.update(old_self, updated_self)?;
                  }
                };
              }
              None => {}
            };
          }
        };
      }
      None => {}
    };
    Ok(())
  }
  pub(crate) fn mark_book_path_as_deleted(book_size: BookSize, target_book_path: &BookPath, db: &DB) -> anyhow::Result<()> {
    match Self::get(book_size, db)? {
      Some(old_self) => {
        let mut updated_self = old_self.clone();
        match &mut updated_self.book_type {
          BookType::UniqueSize { book_path, mutool_data: _ } => {
            match target_book_path.eq(book_path) {
              true => {
                book_path.deleted = true;
                db.update(old_self, updated_self)?;
              }
              false => {}
            };
          }
          BookType::DuplicateSize(hash_map) => {
            match hash_map.swap_remove(target_book_path) {
              Some(data) => {
                match &data {
                  DuplicateBookData::BookHash(book_hash) => {
                    BookHashes::mark_book_as_deleted(book_hash.clone(), target_book_path, db)?;
                  }
                  DuplicateBookData::MutoolData(_mutool_data) => {}
                };
                let mut new_path = target_book_path.clone();
                new_path.deleted = true;
                hash_map.insert(new_path, data);
                // book_path.deleted = true;
                db.update(old_self, updated_self)?;
              }
              None => {}
            };
          }
        };
      }
      None => {}
    };
    Ok(())
  }
  pub(crate) fn update_book_path(book_size: BookSize, old_book_path: &BookPath, new_book_path: BookPath, db: &DB) -> anyhow::Result<()> {
    match Self::get(book_size, db)? {
      Some(old_self) => {
        let mut updated_self = old_self.clone();
        match &mut updated_self.book_type {
          BookType::UniqueSize { book_path, mutool_data: _ } => {
            match old_book_path.eq(book_path) {
              true => {
                *book_path = new_book_path;
                db.update(old_self, updated_self)?;
              }
              false => {}
            };
          }
          BookType::DuplicateSize(hash_map) => {
            match hash_map.swap_remove(old_book_path) {
              Some(data) => {
                match &data {
                  DuplicateBookData::BookHash(book_hash) => {
                    BookHashes::update_book_path(book_hash.clone(), old_book_path, &new_book_path, db)?;
                  }
                  DuplicateBookData::MutoolData(_mutool_data) => {}
                };
                hash_map.insert(new_book_path, data);
                db.update(old_self, updated_self)?;
              }
              None => {}
            };
          }
        };
      }
      None => {}
    };
    Ok(())
  }
}
