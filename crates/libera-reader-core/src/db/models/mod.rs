pub mod settings;
pub mod lang;
pub mod theme;
pub mod route;
pub mod book_data_pk;
pub mod data_of_hashed_book;
pub mod book_data;
pub mod data_of_unhashed_book;
pub mod book_mark;
pub mod book;

use crate::db::models::book_data::BookData;
use crate::db::DB;
use anyhow::Result;
pub use book::Book;
pub use book_data_pk::BookDataPK;
pub use book_mark::BookMark;
pub use data_of_hashed_book::DataOfHashedBook;
pub use data_of_unhashed_book::DataOfUnhashedBook;
pub use lang::*;
use native_db::{ToInput, ToKey};
pub use route::{RootRoute, Route};
pub use settings::SettingsModel;
pub use theme::{ColorScheme, Theme, ThemeData};

pub(crate) trait GetBookData {
  fn get_book_data(&self) -> &BookData;
  fn get_book_data_mut(&mut self) -> &mut BookData;
}
pub(crate) trait DefaultModel {
  fn default_model() -> Self where Self: Sized + ToInput;
}
pub(crate) trait GetOrCreate: Sized + ToInput + Clone + DefaultModel {
  fn get_or_create(key: impl ToKey, db: &DB) -> Result<Self> {
    Ok(db.get_primary::<Self>(key)?.unwrap_or_else(|| {
      let item: Self = Self::default_model();
      db.insert(item.clone()).unwrap();
      item
    }))
  }
}
