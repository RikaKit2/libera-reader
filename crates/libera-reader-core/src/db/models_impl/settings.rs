use crate::db::crud;
use crate::db::models::{Lang, RootRoute, Route, Settings, Theme};
use crate::db::models_impl::{DefaultModel, GetOrCreate};
use crate::types::DB;
use native_db::{Database, ToInput};
use tracing::debug;

impl Settings {
  pub fn new(db: &Database) -> Self { Self::get_or_create(1, db) }
  pub fn create_if_not_exist(db: &Database) { Self::get_or_create(1, db); }
  pub fn set_path_to_scan(&mut self, new_path_to_scan: String, db: &DB) {
    match &self.path_to_scan {
      None => { self.set_path_to_scan_inn(new_path_to_scan, db); }
      Some(old_path_to_scan) => {
        match &old_path_to_scan.eq(&new_path_to_scan) {
          true => {}
          false => {
            let curr_path_eq_with_new = &new_path_to_scan == old_path_to_scan;
            match curr_path_eq_with_new {
              true => { debug!("old path to scan eq with old path to scan"); }
              false => { self.set_path_to_scan_inn(new_path_to_scan, db); }
            };
          }
        }
      }
    }
  }
  fn set_path_to_scan_inn(&mut self, new_path_to_scan: String, db: &DB) {
    Self::set_path_to_scan_db_only(new_path_to_scan.clone(), Some(self.clone()), db);
    debug!("new path to scan: {:?}", &new_path_to_scan);
    self.path_to_scan = Some(new_path_to_scan);
  }
  pub fn set_language(&mut self, new_language: Lang, db: &DB) {
    match &self.language.eq(&new_language) {
      true => {}
      false => {
        Self::set_language_db_only(new_language.clone(), Some(self.clone()), db);
        self.language = new_language;
      }
    };
  }
  pub fn set_theme(&mut self, theme: Theme, db: &DB) {
    match &self.theme.eq(&theme) {
      true => {}
      false => {
        Self::set_theme_db_only(theme.clone(), Some(self.clone()), db);
        self.theme = theme
      }
    };
  }
  pub fn set_route(&mut self, new_route: RootRoute, db: &DB) {
    match &self.route.eq(&new_route) {
      true => {}
      false => {
        Self::set_route_db_only(new_route.clone(), Some(self.clone()), db);
        self.route = new_route;
      }
    }
  }
  pub fn compare_route_with_other(&self, other: &Route) -> bool {
    match &self.route {
      RootRoute::Main(base_route) => { other.eq(base_route) }
      RootRoute::BookViewer => { false }
      RootRoute::Setup => { false }
    }
  }
  pub fn set_setup_status(&mut self, status: bool, db: &DB) {
    match &self.setup_is_done.eq(&status) {
      true => {}
      false => {
        Self::set_setup_status_db_only(status, Some(self.clone()), db);
        self.setup_is_done = status;
      }
    }
  }
  pub fn set_path_to_scan_db_only(new_path_to_scan: String, old_settings: Option<Self>, db: &DB) {
    Self::update_settings(db, old_settings, |new_settings| new_settings.path_to_scan = Some(new_path_to_scan));
  }
  pub fn set_language_db_only(new_language: Lang, old_settings: Option<Self>, db: &DB) {
    Self::update_settings(db, old_settings, |new_settings| new_settings.language = new_language);
  }
  pub fn set_theme_db_only(theme: Theme, old_settings: Option<Self>, db: &DB) {
    Self::update_settings(db, old_settings, |new_settings| new_settings.theme = theme);
  }
  pub fn get_path_to_scan_db_only(db: &DB) -> Option<String> {
    Self::get_self(db).path_to_scan
  }
  pub fn set_route_db_only(route: RootRoute, old_settings: Option<Self>, db: &DB) {
    Self::update_settings(db, old_settings, |new_settings| new_settings.route = route)
  }
  pub fn set_setup_status_db_only(status: bool, old_settings: Option<Self>, db: &DB) {
    Self::update_settings(db, old_settings, |new_settings| new_settings.setup_is_done = status)
  }
  pub fn get_self(db: &DB) -> Self {
    crud::get_primary::<Self>(1, db).unwrap()
  }
  // old settings must be actual, call this function until changing passed settings
  fn update_settings<F>(db: &DB, old_settings: Option<Self>, changing_fn: F) where F: FnOnce(&mut Settings) {
    let old_settings = match old_settings {
      None => { crud::get_primary::<Self>(1, db).unwrap() }
      Some(res) => { res }
    };
    let mut new_settings = old_settings.clone();
    changing_fn(&mut new_settings);
    crud::update::<Self>(old_settings, new_settings, db).unwrap();
  }
}
impl DefaultModel for Settings {
  fn default_model() -> Self where Self: Sized + ToInput {
    Self {
      id: 1,
      language: Lang::detect_system_lang(),
      theme: Theme::make_sunset(),
      path_to_scan: None,
      number_of_columns: 6,
      page_scaling_factor: 1.0,
      thumbnails_scaling_factor: 4.0,
      workers_num: 2,
      route: RootRoute::Setup,
      setup_is_done: false,
    }
  }
}
impl GetOrCreate for Settings {}
