use crate::db::crud;
use crate::db::models_impl::{GetOrCreate, NewModel};
use crate::models::{Language, Settings, Theme};
use crate::services::notify_service;
use native_db::ToInput;
use tracing::debug;

impl Settings {
  pub(crate) fn new() -> Self { Self::get_or_create(1) }
  pub fn set_path_to_scan(&mut self, new_path_to_scan: String) {
    debug!("set path to scan: {:?}", &new_path_to_scan);
    self.path_to_scan = Some(new_path_to_scan.clone());
    match &self.path_to_scan {
      None => { self.simple_set_path_to_scan(new_path_to_scan); }
      Some(old_path_to_scan) => {
        let curr_path_eq_with_new = &new_path_to_scan == old_path_to_scan;
        match curr_path_eq_with_new {
          true => {}
          false => {
            notify_service::stop_watcher(old_path_to_scan);
            self.simple_set_path_to_scan(new_path_to_scan);
          }
        };
      }
    }
  }
  fn simple_set_path_to_scan(&mut self, new_path_to_scan: String) {
    notify_service::run_watcher(&new_path_to_scan).unwrap();
    let old_settings = self.clone();
    let mut new_settings = old_settings.clone();
    new_settings.path_to_scan = Some(new_path_to_scan);
    crud::update::<Self>(old_settings, new_settings).unwrap();
  }
}

impl Default for Settings {
  fn default() -> Self {
    Self::new()
  }
}
impl NewModel for Settings {
  fn new_model() -> Self
  where
    Self: Sized + ToInput,
  {
    Self {
      id: 1,
      language: Language::EN,
      theme: Theme::Sunset,
      path_to_scan: None,
      number_of_columns: 6,
      page_scaling_factor: 1.0,
      thumbnails_scaling_factor: 4.0,
      workers_num: 2,
    }
  }
}
impl GetOrCreate for Settings {}
