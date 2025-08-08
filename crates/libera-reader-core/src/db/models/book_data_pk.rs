use crate::db::models::book_data::BookData;
use crate::db::models::data_of_hashed_book::DataOfHashedBook;
use crate::db::models::data_of_unhashed_book::DataOfUnhashedBook;
use crate::types::{BookHash, BookSize, DB};
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub enum BookDataPK {
  UniqueSize(BookSize),    // DataOfHashedBook
  RepeatingSize(BookHash), // DataOfUnhashedBook
}
impl BookDataPK {
  pub(crate) fn update<F>(self, book_data_change_func: F, db: &DB) -> Result<()>
                          where F: FnOnce(&mut BookData) {
    match self {
      BookDataPK::UniqueSize(book_size) => {
        let wrapper = db.get_primary::<DataOfUnhashedBook>(book_size)?.unwrap();
        let mut new_wrapper = wrapper.clone();
        book_data_change_func(&mut new_wrapper.book_data);
        db.update(wrapper, new_wrapper)?;
      }
      BookDataPK::RepeatingSize(book_hash) => {
        let wrapper = db.get_primary::<DataOfHashedBook>(book_hash)?.unwrap();
        let mut new_wrapper = wrapper.clone();
        book_data_change_func(&mut new_wrapper.book_data);
        db.update(wrapper, new_wrapper)?;
      }
    }
    Ok(())
  }
}
