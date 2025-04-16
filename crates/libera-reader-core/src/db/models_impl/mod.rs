pub(crate) mod book;
pub(crate) mod settings;
mod target_ext;
mod data_of_unhashed_book;
mod data_of_hashed_book;
mod book_data;

use crate::db::crud::{get_primary, insert};
use crate::db::models::BookData;
use native_db::{ToInput, ToKey};


pub trait GetBookData {
  fn get_book_data_as_ref(&self) -> &BookData;
}
pub trait NewModel {
  fn new_model() -> Self
  where
    Self: Sized + ToInput;
}
pub trait GetOrCreate: Sized + ToInput + Clone + NewModel {
  fn get_or_create(key: impl ToKey) -> Self {
    get_primary::<Self>(key).unwrap_or_else(|| {
      let item: Self = Self::new_model();
      insert(item.clone()).unwrap();
      item
    })
  }
}

