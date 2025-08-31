use crate::db::DB;
use crate::db::models::book_data::BookData;
use crate::db::models::data_of_hashed_book::DataOfHashedBook;
use crate::db::models::data_of_unhashed_book::DataOfUnhashedBook;
use crate::types::{BookHash, BookSize};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub enum BookDataPK {
  UniqueSize(BookSize),
  Hashed(BookHash),
}
impl BookDataPK {
  pub(crate) fn update<F>(self, book_data_change_func: F, db: &DB) -> Result<(), Box<dyn std::error::Error>>
  where
    F: FnOnce(&mut BookData),
  {
    let res: Result<(), Box<dyn std::error::Error>> = match self {
      BookDataPK::UniqueSize(book_size) => match db.get_primary::<DataOfUnhashedBook>(book_size.clone())? {
        None => Err(Box::from(book_size.to_string())),
        Some(wrapper) => {
          let mut new_wrapper = wrapper.clone();
          book_data_change_func(&mut new_wrapper.book_data);
          db.update(wrapper, new_wrapper)?;
          Ok(())
        }
      },
      BookDataPK::Hashed(book_hash) => match db.get_primary::<DataOfHashedBook>(book_hash.clone())? {
        None => Err(Box::from(book_hash)),
        Some(wrapper) => {
          let mut new_wrapper = wrapper.clone();
          book_data_change_func(&mut new_wrapper.book_data);
          db.update(wrapper, new_wrapper)?;
          Ok(())
        }
      },
    };
    res
  }
}
