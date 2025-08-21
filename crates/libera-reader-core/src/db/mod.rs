pub mod models;

use crate::db::models::book::Book;
use crate::db::models::book_mark::BookMark;
use crate::db::models::data_of_hashed_book::DataOfHashedBook;
use crate::db::models::data_of_unhashed_book::DataOfUnhashedBook;
use crate::db::models::settings::SettingsModel;
use anyhow::Result;
use itertools::Itertools;
use native_db::{db_type, Builder, Database, Models, ToInput, ToKey};
use once_cell::sync::Lazy;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use sysinfo::System;
use tracing::info;

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

pub(crate) enum DB {
  InMemory(Database<'static>),
  InFile(Database<'static>),
}
impl DB {
  pub(crate) fn new_in_memory() -> Self {
    Self::InMemory(Builder::new().create_in_memory(&MODELS).unwrap())
  }
  pub(crate) fn new_in_file(path_to_db: &PathBuf) -> Self {
    Self::InFile(Builder::new().open(&MODELS, path_to_db).unwrap())
  }
  pub(crate) fn no_mut(&self) -> &Database<'static> {
    match self {
      Self::InMemory(inn) => inn,
      Self::InFile(inn) => inn,
    }
  }
  pub(crate) fn mut_only(&mut self) -> &mut Database<'static> {
    match self {
      Self::InMemory(inn) => inn,
      Self::InFile(inn) => inn,
    }
  }
}
#[derive(Clone)]
pub struct DataBase {
  inn: Arc<RwLock<DB>>,
  path_to_db: Arc<RwLock<PathBuf>>,
}
impl DataBase {
  pub fn new(path_to_db: PathBuf) -> Result<Self> {
    let db = match path_to_db.exists() {
      true => { DB::new_in_file(&path_to_db) }
      false => { DB::new_in_memory() }
    };
    Ok(Self {
      inn: Arc::new(RwLock::new(db)),
      path_to_db: Arc::new(RwLock::new(path_to_db)),
    })
  }
  fn read(&self) -> RwLockReadGuard<'_, DB> {
    self.inn.read().unwrap()
  }
  fn write(&self) -> RwLockWriteGuard<'_, DB> {
    self.inn.write().unwrap()
  }
  //noinspection RsUnwrap
  pub(crate) fn save_to_storage(&self) -> Result<()> {
    match self.inn.read().unwrap().deref() {
      DB::InMemory(db) => { db.snapshot(&MODELS, &self.path_to_db.read().unwrap())?; }
      DB::InFile(_) => {}
    }
    Ok(())
  }
  //noinspection RsUnwrap
  pub(crate) fn reload_db(&self) -> Result<()> {
    let db_in_memory = match self.read().deref() {
      DB::InMemory(_) => true,
      DB::InFile(_) => false,
    };

    if db_in_memory {
      let pid = sysinfo::get_current_pid().unwrap();
      let mut sys = System::new_all();

      sys.refresh_all();
      let before = sys.process(pid).map(|p| p.memory() as f64 / (1024.0 * 1024.0)).unwrap_or(0.0);

      let mut db = self.inn.write().unwrap();
      *db.deref_mut() = DB::new_in_file(&self.path_to_db.read().unwrap().clone());

      sys.refresh_all();
      let after = sys.process(pid).map(|p| p.memory() as f64 / (1024.0 * 1024.0)).unwrap_or(0.0);

      info!("DB reloaded successfully; db in memory, usage: {:.2} MB; db on disk, usage: {:.2} MB; Memory {:.2} MB is released", before, after, before - after);
    }
    Ok(())
  }
  pub fn compact(&self) -> Result<()> {
    self.write().mut_only().compact()?;
    Ok(())
  }
  pub(crate) fn get_primary<T: ToInput>(&self, key: impl ToKey) -> Result<Option<T>> {
    Ok(self.read().no_mut().r_transaction()?.get().primary(key)?)
  }
  pub(crate) fn scan_primary<T: ToInput>(&self) -> Result<Vec<T>> {
    Ok(self.read().no_mut().r_transaction()?.scan().primary()?.all()?.try_collect()?)
  }
  pub(crate) fn scan_primary_by<Key: ToKey, Table: ToInput>(&self, key_data: Key) -> Result<Vec<Table>> {
    Ok(self.read().no_mut().r_transaction()?.scan().primary()?.start_with(key_data)?.try_collect()?)
  }
  pub(crate) fn insert<T: ToInput>(&self, item: T) -> db_type::Result<()> {
    let lock = self.write();
    let rw_conn = lock.no_mut().rw_transaction()?;
    rw_conn.insert(item)?;
    rw_conn.commit()
  }
  pub(crate) fn update<T: ToInput>(&self, old_data: T, new_data: T) -> db_type::Result<()> {
    let lock = self.write();
    let rw_conn = lock.no_mut().rw_transaction()?;
    rw_conn.update(old_data, new_data)?;
    rw_conn.commit()
  }
  pub(crate) fn remove<T: ToInput>(&self, item: T) -> db_type::Result<()> {
    let lock = self.write();
    let rw_conn = lock.no_mut().rw_transaction()?;
    rw_conn.remove(item)?;
    rw_conn.commit()
  }
}
