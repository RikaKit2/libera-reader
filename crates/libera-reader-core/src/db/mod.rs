pub mod models;

use crate::db::models::books::Books;
use crate::db::models::books::book_hashes::BookHashes;
use crate::db::models::books::book_sizes::BookSizes;
use crate::db::models::settings::Settings;
use anyhow::Result;
use itertools::Itertools;
use native_db::transaction::{RTransaction, RwTransaction};
use native_db::{Builder, Database, Models, ToInput, ToKey, db_type};
use once_cell::sync::Lazy;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

fn get_models() -> Result<Models> {
  let mut models = Models::new();
  models.define::<Books>()?;
  models.define::<Settings>()?;
  models.define::<BookSizes>()?;
  models.define::<BookHashes>()?;
  Ok(models)
}

static MODELS: Lazy<Models> = Lazy::new(|| get_models().unwrap());

#[derive(Clone)]
pub struct DB {
  db: Arc<RwLock<Database<'static>>>,
}

impl DB {
  pub fn new(path_to_db: PathBuf) -> Result<Self> {
    let db = if path_to_db.exists() {
      Builder::new().open(&MODELS, &path_to_db)?
    } else {
      Builder::new().create(&MODELS, &path_to_db)?
    };
    Ok(Self { db: Arc::new(RwLock::new(db)) })
  }

  pub fn compact(&self) -> Result<()> {
    self.db.write().unwrap().compact()?;
    Ok(())
  }

  pub(crate) fn get_primary<T: ToInput>(&self, key: impl ToKey) -> Result<Option<T>> {
    Ok(self.db.read().unwrap().r_transaction()?.get().primary(key)?)
  }

  pub(crate) fn insert<T: ToInput>(&self, item: T) -> Result<(), Box<db_type::Error>> {
    let lock = self.db.write().unwrap();
    let rw_conn = lock.rw_transaction().map_err(Box::new)?;
    rw_conn.insert(item).map_err(Box::new)?;
    rw_conn.commit().map_err(Box::new)
  }

  pub(crate) fn update<T: ToInput>(&self, old_data: T, new_data: T) -> Result<(), Box<db_type::Error>> {
    let lock = self.db.write().unwrap();
    let rw_conn = lock.rw_transaction().map_err(Box::new)?;
    rw_conn.update(old_data, new_data).map_err(Box::new)?;
    rw_conn.commit().map_err(Box::new)
  }

  pub fn rw_t<F, T>(&self, func: F) -> Result<T>
  where
    F: FnOnce(&RwTransaction) -> Result<T>,
  {
    let lock = self.db.write().unwrap();
    let rw_txn = lock.rw_transaction()?;
    let result = func(&rw_txn)?;
    rw_txn.commit()?;
    Ok(result)
  }

  pub fn rt<F, T>(&self, func: F) -> Result<T>
  where
    F: FnOnce(&RTransaction) -> Result<T>,
  {
    let lock = self.db.write().unwrap();
    let r_txn = lock.r_transaction()?;
    let result = func(&r_txn)?;
    Ok(result)
  }
}

pub(crate) fn scan_primary<T: ToInput>(r: &RTransaction<'_>) -> Result<Vec<T>> {
  Ok(r.scan().primary()?.all()?.try_collect()?)
}
