pub mod settings;
pub mod lang;
pub mod theme;
pub mod route;
pub mod book_data_wrapper_pk;
pub mod data_of_hashed_book;
pub mod book_data;
pub mod data_of_unhashed_book;
pub mod book_mark;
pub mod book;
pub mod target_ext;

use crate::db::crud::{get_primary, insert};
use crate::db::models::book_data::BookData;
use anyhow::Result;
pub use book::Book;
pub use book_data_wrapper_pk::BookDataWrapperPK;
pub use book_mark::BookMark;
pub use data_of_hashed_book::DataOfHashedBook;
pub use data_of_unhashed_book::DataOfUnhashedBook;
pub use lang::*;
use native_db::{Database, ToInput, ToKey};
pub use route::{RootRoute, Route};
pub use settings::Settings;
pub use target_ext::TargetExt;
pub use theme::{ColorScheme, Theme, ThemeData};


pub(crate) trait GetBookData {
  fn get_book_data_as_ref(&self) -> &BookData;
}
pub trait DefaultModel {
  fn default_model() -> Self
                     where Self: Sized + ToInput;
}
pub trait GetOrCreate: Sized + ToInput + Clone + DefaultModel {
  fn get_or_create(key: impl ToKey, db: &Database) -> Result<Self> {
    Ok(get_primary::<Self>(key, db)?.unwrap_or_else(|| {
      let item: Self = Self::default_model();
      insert(item.clone(), db).unwrap();
      item
    }))
  }
}

