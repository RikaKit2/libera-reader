use crate::db::{crud, models, DB};
use crossbeam_channel::Receiver;
use gxhash::{HashSet, HashSetExt};
use native_db::db_type;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::ops::Deref;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::error;


pub(crate) type TargetExtensions = Arc<RwLock<HashSet<String>>>;
pub struct Settings {
  path_to_scan: Arc<RwLock<Option<String>>>,
  target_ext: TargetExtensions,
  watcher: RecommendedWatcher,
  pub(crate) notify_receiver: Receiver<notify::Result<notify::Event>>,
}

impl Settings {
  pub fn new() -> Result<Settings, db_type::Error> {
    match Settings::get_settings_model() {
      Ok(settings) => {
        let (tx, notify_receiver) = crossbeam_channel::unbounded();
        let watcher = notify::recommended_watcher(move |res| { tx.send(res).unwrap(); }).unwrap();
        let mut target_ext: HashSet<String> = HashSet::new();
        if settings.epub {
          target_ext.insert("epub".to_string());
        };
        if settings.pdf {
          target_ext.insert("pdf".to_string());
        };
        if settings.mobi {
          target_ext.insert("mobi".to_string());
        };
        Ok(Self {
          path_to_scan: Arc::new(RwLock::new(settings.path_to_scan)),
          target_ext: Arc::new(RwLock::new(target_ext)),
          watcher,
          notify_receiver,
        })
      }
      Err(e) => {
        Err(e)
      }
    }
  }
  pub async fn set_path_to_scan(&mut self, path_to_scan: String) {
    let old_settings = Settings::get_settings_model().unwrap();

    match &old_settings.path_to_scan {
      None => {}
      Some(path_to_scan) => {
        self.stop_notify(path_to_scan).await;
      }
    };
    self.run_notify(&path_to_scan).await;
    let mut new_settings = old_settings.clone();
    new_settings.path_to_scan = Some(path_to_scan.clone());
    match crud::update::<models::Settings>(old_settings, new_settings) {
      Ok(_) => {}
      Err(e) => {
        error!("{:?}", &e);
        panic!("{:?}", e);
      }
    };

    let _ = self.path_to_scan.write().await.insert(path_to_scan);
  }
  pub async fn get_path_to_scan(&self) -> Option<String> {
    self.path_to_scan.read().await.deref().clone()
  }
  pub async fn get_path_to_scan_as_link(&self) -> Arc<RwLock<Option<String>>> {
    self.path_to_scan.clone()
  }
  pub async fn path_to_scan_is_valid(&self) -> bool {
    self.path_to_scan.read().await.is_some()
  }
  pub fn get_target_ext(&self) -> Arc<RwLock<HashSet<String>>> {
    self.target_ext.clone()
  }
  pub async fn run_notify(&mut self, path_to_scan: &String) {
    self.watcher.watch(path_to_scan.as_ref(), RecursiveMode::Recursive).unwrap();
  }
  pub async fn stop_notify(&mut self, path_to_scan: &String) {
    match self.watcher.unwatch(path_to_scan.as_ref()) {
      Ok(_) => {}
      Err(_) => {}
    };
  }

  //noinspection RsUnwrap
  fn get_settings_model() -> Result<models::Settings, db_type::Error> {
    let r_conn = DB.r_transaction().unwrap();
    let result: db_type::Result<Option<models::Settings>> = r_conn.get().primary(1i32);
    match result {
      Ok(poss_settings) => {
        match poss_settings {
          None => {
            let settings_model = models::Settings::new();
            crud::insert(settings_model.clone()).unwrap();
            Ok(settings_model)
          }
          Some(settings) => {
            Ok(settings)
          }
        }
      }
      Err(e) => {
        Err(e)
      }
    }
  }
}
