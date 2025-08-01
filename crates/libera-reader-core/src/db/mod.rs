pub mod models;
pub(crate) mod crud;

use crate::db::models::book::Book;
use crate::db::models::book_mark::BookMark;
use crate::db::models::data_of_hashed_book::DataOfHashedBook;
use crate::db::models::data_of_unhashed_book::DataOfUnhashedBook;
use crate::db::models::settings::SettingsModel;
use anyhow::Result;
use native_db::{db_type, Builder, Database, Models};
use std::path::PathBuf;
use tracing::debug;

pub fn create_db_on_disk<'a>(path_to_db: PathBuf, models: &'static Models) -> db_type::Result<Database<'a>> {
  match path_to_db.exists() {
    true => { Builder::new().open(models, path_to_db) }
    false => {
      debug!("Creating a DB");
      Builder::new().create(models, path_to_db)
    }
  }
}
pub fn create_db_in_memory(models: &'static Models) -> Result<Database<'static>> {
  Ok(Builder::new().create_in_memory(models)?)
}

pub fn get_models() -> Result<Models> {
  let mut models = Models::new();
  models.define::<SettingsModel>()?;
  models.define::<BookMark>()?;
  models.define::<Book>()?;
  models.define::<DataOfUnhashedBook>()?;
  models.define::<DataOfHashedBook>()?;
  Ok(models)
}
