use crate::db::{crud, models, DB};
use crate::services::notify_service::watchdog;
use crate::vars::{PATH_TO_SCAN, TARGET_EXT};
use native_db::db_type;
use tracing::error;


pub struct Settings {}

impl Settings {
  pub(crate) fn new() -> Result<Settings, db_type::Error> {
    match Settings::get_settings_model() {
      Ok(settings) => {
        match settings.path_to_scan {
          None => {}
          Some(path_to_scan) => {
            let _ = PATH_TO_SCAN.write().unwrap().insert(path_to_scan);
          }
        }
        if settings.epub {
          TARGET_EXT.write().unwrap().insert("epub".to_string());
        };
        if settings.pdf {
          TARGET_EXT.write().unwrap().insert("pdf".to_string());
        };
        if settings.mobi {
          TARGET_EXT.write().unwrap().insert("mobi".to_string());
        };
        Ok(Self {})
      }
      Err(e) => {
        Err(e)
      }
    }
  }
  pub fn set_path_to_scan(&mut self, path_to_scan: String) {
    let old_settings = Settings::get_settings_model().unwrap();

    match &old_settings.path_to_scan {
      None => {}
      Some(path_to_scan) => {
        watchdog::stop(path_to_scan).unwrap();
      }
    };
    watchdog::run(&path_to_scan);

    let mut new_settings = old_settings.clone();
    new_settings.path_to_scan = Some(path_to_scan.clone());
    match crud::update::<models::Settings>(old_settings, new_settings) {
      Ok(_) => {}
      Err(e) => {
        error!("{:?}", &e);
        panic!("{:?}", e);
      }
    };
    let _ = PATH_TO_SCAN.write().unwrap().insert(path_to_scan);
  }
  pub fn get_path_to_scan(&self) -> Option<String> {
    PATH_TO_SCAN.read().unwrap().clone()
  }
  pub fn path_to_scan_is_valid(&self) -> bool {
    PATH_TO_SCAN.read().unwrap().is_some()
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
impl Default for Settings {
  fn default() -> Self {
    Self::new().unwrap()
  }
}
