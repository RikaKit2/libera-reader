use crate::db::models::{Book, BookMark, DataOfHashedBook, DataOfUnhashedBook, Settings};
use crate::models::TargetExt;
use crate::vars::APP_DIRS;
use native_db::{Builder, Database, Models};
use once_cell::sync::Lazy;


pub mod models;
pub(crate) mod crud;
pub(crate) mod models_impl;

fn get_models() -> Models {
  let mut models = Models::new();
  models.define::<Settings>().unwrap();
  models.define::<BookMark>().unwrap();
  models.define::<Book>().unwrap();
  models.define::<DataOfUnhashedBook>().unwrap();
  models.define::<DataOfHashedBook>().unwrap();
  models.define::<TargetExt>().unwrap();
  models
}

fn get_db(models: &Models) -> native_db::db_type::Result<Database> {
  let path_to_db = &APP_DIRS.read().unwrap().path_to_db;
  match path_to_db.exists() {
    true => { Builder::new().open(models, path_to_db) }
    false => { Builder::new().create(models, path_to_db) }
  }
}

static MODELS: Lazy<Models> = Lazy::new(|| get_models());
pub(crate) static DB: Lazy<Database> = Lazy::new(|| get_db(&MODELS).unwrap());
