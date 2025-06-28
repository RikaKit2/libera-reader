pub mod models;
pub mod models_impl;
pub(crate) mod crud;

use crate::db::models::{Book, BookMark, DataOfHashedBook, DataOfUnhashedBook, Settings, TargetExt, Theme};
use native_db::{db_type, Builder, Database, Models};
use once_cell::sync::Lazy;
use std::path::PathBuf;
use tracing::debug;


pub fn create_db_on_disk<'a>(path_to_db: PathBuf) -> db_type::Result<Database<'a>> {
  match path_to_db.exists() {
    true => { Builder::new().open(&MODELS, path_to_db) }
    false => {
      debug!("Creating a DB");
      Builder::new().create(&MODELS, path_to_db)
    }
  }
}
pub fn create_db_in_memory() -> Database<'static> {
  Builder::new().create_in_memory(&MODELS).unwrap()
}
pub(crate) fn get_models() -> Models {
  let mut models = Models::new();
  models.define::<Settings>().unwrap();
  models.define::<BookMark>().unwrap();
  models.define::<Book>().unwrap();
  models.define::<DataOfUnhashedBook>().unwrap();
  models.define::<DataOfHashedBook>().unwrap();
  models.define::<TargetExt>().unwrap();
  models.define::<Theme>().unwrap();
  models
}
pub static MODELS: Lazy<Models> = Lazy::new(|| get_models());
