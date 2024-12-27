use crate::db::models::{Book, BookMark, DataOfHashedBook, DataOfUnhashedBook, Settings};
use native_db::{Builder, Database, Models};
use once_cell::sync::Lazy;
use std::path::Path;

pub(crate) mod models;
pub(crate) mod crud;
pub(crate) mod models_impl;

fn get_models() -> Models {
  let mut models = Models::new();
  models.define::<Settings>().unwrap();
  models.define::<BookMark>().unwrap();
  models.define::<Book>().unwrap();
  models.define::<DataOfUnhashedBook>().unwrap();
  models.define::<DataOfHashedBook>().unwrap();
  models
}

fn get_db(models: &Models) -> native_db::db_type::Result<Database> {
  let db_name = "libera_reader.redb";
  match Path::new(db_name).exists() {
    true => {
      Builder::new().open(models, db_name)
    }
    false => {
      Builder::new().create(models, db_name)
    }
  }
}

static MODELS: Lazy<Models> = Lazy::new(|| get_models());
pub(crate) static DB: Lazy<Database> = Lazy::new(|| get_db(&MODELS).unwrap());
