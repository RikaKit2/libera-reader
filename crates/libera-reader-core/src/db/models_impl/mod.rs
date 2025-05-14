pub(crate) mod book;
pub(crate) mod settings;
mod target_ext;
mod data_of_unhashed_book;
mod data_of_hashed_book;
mod book_data;
mod book_data_wrapper_pk;
use crate::db::crud::{get_primary, insert};
use crate::db::models::BookData;
use native_db::{Database, ToInput, ToKey};

pub(crate) trait GetBookData {
  fn get_book_data_as_ref(&self) -> &BookData;
}
pub trait DefaultModel {
  fn default_model() -> Self where Self: Sized + ToInput;
}
pub trait GetOrCreate: Sized + ToInput + Clone + DefaultModel {
  fn get_or_create(key: impl ToKey, db: &Database) -> Self {
    get_primary::<Self>(key, db).unwrap_or_else(|| {
      let item: Self = Self::default_model();
      insert(item.clone(), db).unwrap();
      item
    })
  }
}

