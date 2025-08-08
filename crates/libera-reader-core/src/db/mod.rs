pub mod models;

use crate::db::models::book::Book;
use crate::db::models::book_mark::BookMark;
use crate::db::models::data_of_hashed_book::DataOfHashedBook;
use crate::db::models::data_of_unhashed_book::DataOfUnhashedBook;
use crate::db::models::settings::SettingsModel;
use anyhow::Result;
use itertools::Itertools;
use native_db::db_type::{KeyOptions, ToKeyDefinition};
use native_db::{db_type, Builder, Database, Models, ToInput, ToKey};
use once_cell::sync::Lazy;
use std::path::PathBuf;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

fn get_models() -> Result<Models> {
  let mut models = Models::new();
  models.define::<SettingsModel>()?;
  models.define::<BookMark>()?;
  models.define::<Book>()?;
  models.define::<DataOfUnhashedBook>()?;
  models.define::<DataOfHashedBook>()?;
  Ok(models)
}

static MODELS: Lazy<Models> = Lazy::new(|| get_models().unwrap());

#[derive(Clone)]
pub struct DataBase {
  inn: Arc<RwLock<Database<'static>>>,
  path_to_db: Arc<RwLock<PathBuf>>,
}
impl DataBase {
  pub fn new(path_to_db: PathBuf) -> Result<Self> {
    let db_exists = path_to_db.exists();
    let db = match db_exists {
      true => { Builder::new().create(&MODELS, &path_to_db)? }
      false => { Builder::new().create_in_memory(&MODELS)? }
    };
    Ok(Self {
      inn: Arc::new(RwLock::new(db)),
      path_to_db: Arc::new(RwLock::new(path_to_db)),
    })
  }
  pub(crate) fn read(&self) -> RwLockReadGuard<'_, Database<'static>> {
    self.inn.read().unwrap()
  }
  fn write(&self) -> RwLockWriteGuard<'_, Database<'static>> {
    self.inn.write().unwrap()
  }
  //noinspection RsUnwrap
  pub(crate) fn save_to_storage(&self) -> Result<()> {
    let path_to_db = self.path_to_db.read().unwrap();
    match &path_to_db.as_path().exists() {
      true => { std::fs::remove_file(&path_to_db.as_path())?; }
      false => {}
    }
    self.write().snapshot(&MODELS, &path_to_db)?;
    Ok(())
  }
  pub(crate) fn reload_db(&self) -> Result<()> {
    *self.write() = Builder::new().open(&MODELS, &self.path_to_db.read().unwrap().as_path())?;
    Ok(())
  }
  pub fn compact(&self) -> Result<()> {
    self.write().compact()?;
    Ok(())
  }
  pub(crate) fn get_primary<T: ToInput>(&self, key: impl ToKey) -> Result<Option<T>> {
    Ok(self.read().r_transaction()?.get().primary(key)?)
  }
  pub(crate) fn scan_primary<T: ToInput>(&self) -> Result<Vec<T>> {
    Ok(self.read().r_transaction()?.scan().primary()?.all()?.try_collect()?)
  }
  pub(crate) fn scan_primary_by<Key: ToKey, Table: ToInput>(&self, key_data: Key) -> Result<Vec<Table>> {
    Ok(self.read().r_transaction()?.scan().primary()?.start_with(key_data)?.try_collect()?)
  }
  pub(crate) fn scan_secondary_by<Key: ToKey, Table: ToInput>(&self, key_data: Key, key_def: impl ToKeyDefinition<KeyOptions>) -> Result<Vec<Table>> {
    Ok(self.read().r_transaction()?.scan().secondary(key_def)?.start_with(key_data)?.try_collect()?)
  }
  pub(crate) fn insert<T: ToInput>(&self, item: T) -> db_type::Result<()> {
    let db = self.write();
    let rw_conn = db.rw_transaction()?;
    rw_conn.insert(item)?;
    rw_conn.commit()
  }
  pub(crate) fn update<T: ToInput>(&self, old_data: T, new_data: T) -> db_type::Result<()> {
    let db = self.write();
    let rw_conn = db.rw_transaction()?;
    rw_conn.update(old_data, new_data)?;
    rw_conn.commit()
  }
  pub(crate) fn remove<T: ToInput>(&self, item: T) -> db_type::Result<()> {
    let db = self.write();
    let rw_conn = db.rw_transaction()?;
    rw_conn.remove(item)?;
    rw_conn.commit()
  }
}
