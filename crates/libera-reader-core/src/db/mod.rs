pub mod models;
pub mod models_impl;
pub(crate) mod crud;

use crate::db::models::{Book, BookMark, DataOfHashedBook, DataOfUnhashedBook, Settings, TargetExt, Theme};
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
  models.define::<Settings>()?;
  models.define::<BookMark>()?;
  models.define::<Book>()?;
  models.define::<DataOfUnhashedBook>()?;
  models.define::<DataOfHashedBook>()?;
  models.define::<TargetExt>()?;
  models.define::<Theme>()?;
  Ok(models)
}
