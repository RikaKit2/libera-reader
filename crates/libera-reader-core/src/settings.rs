use crate::db::{models, DB};
use std::sync::Arc;
use tokio::sync::{RwLock, RwLockReadGuard};


pub struct Settings {
  path_to_scan: Arc<RwLock<Option<String>>>,
}


impl Settings {
  pub fn new() -> Self {
    Self { path_to_scan: Arc::new(RwLock::new(Settings::get_path_to_scan2())) }
  }
  pub async fn set_path_to_scan(&mut self, path_to_scan: String) {
    let rw_conn = DB.rw_transaction().unwrap();
    let settings_model = Settings::get_self_model();
    let mut new_instance = settings_model.clone();
    new_instance.path_to_scan = Some(path_to_scan.clone());
    rw_conn.update(settings_model, new_instance).unwrap();
    let _ = self.path_to_scan.write().await.insert(path_to_scan);
  }
  pub async fn get_path_to_scan(&self) -> RwLockReadGuard<'_, Option<String>> {
    self.path_to_scan.read().await
  }
  pub fn get_path_to_scan_as_link(&self) -> Arc<RwLock<Option<String>>> {
    self.path_to_scan.clone()
  }
  pub async fn path_to_scan_is_valid(&self) -> bool {
    self.path_to_scan.read().await.is_none()
  }
  fn get_self_model() -> models::Settings {
    let r_conn = DB.r_transaction().unwrap();
    let settings: models::Settings = r_conn.get().primary(1).unwrap().unwrap();
    settings
  }
  fn get_path_to_scan2() -> Option<String> {
    Settings::get_self_model().path_to_scan
  }
}

