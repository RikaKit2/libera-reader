use crate::db::crud;
use crate::db::crud::update_table;
use crate::db::models::{Lang, RootRoute, Route, Settings};
use crate::db::models_impl::{DefaultModel, GetOrCreate};
use crate::types::DB;
use anyhow::Result;
use native_db::ToInput;
use tracing::debug;

impl Settings {
  pub fn new(db: &DB) -> Result<Self> { Self::get_or_create(1, db) }
  pub fn create_if_not_exist(db: &DB) -> Result<()> {
    Self::get_or_create(1, db)?;
    Ok(())
  }
  pub fn set_path_to_scan(&mut self, new_path_to_scan: String, db: &DB) -> Result<()> {
    match &self.path_to_scan {
      None => { self.set_path_to_scan_inn(new_path_to_scan, db)?; }
      Some(old_path_to_scan) => {
        match &old_path_to_scan.eq(&new_path_to_scan) {
          true => {}
          false => {
            let curr_path_eq_with_new = &new_path_to_scan == old_path_to_scan;
            match curr_path_eq_with_new {
              true => { debug!("old path to scan eq with old path to scan"); }
              false => { self.set_path_to_scan_inn(new_path_to_scan, db)?; }
            };
          }
        }
      }
    }
    Ok(())
  }
  fn set_path_to_scan_inn(&mut self, new_path_to_scan: String, db: &DB) -> Result<()> {
    update_table(db, Some(self.clone()), |new_settings| new_settings.path_to_scan = Some(new_path_to_scan.clone()))?;
    debug!("new path to scan: {:?}", &new_path_to_scan);
    self.path_to_scan = Some(new_path_to_scan);
    Ok(())
  }
  pub fn set_language(&mut self, new_language: Lang, db: &DB) -> Result<()> {
    match &self.language.eq(&new_language) {
      true => {}
      false => {
        update_table(db, Some(self.clone()), |new_settings| new_settings.language = new_language.clone())?;
        self.language = new_language;
      }
    };
    Ok(())
  }
  pub fn set_route(&mut self, new_route: RootRoute, db: &DB) -> Result<()> {
    match &self.route.eq(&new_route) {
      true => {}
      false => {
        update_table(db, Some(self.clone()), |new_settings| new_settings.route = new_route.clone())?;
        self.route = new_route;
      }
    }
    Ok(())
  }
  pub fn compare_route_with_other(&self, other: &Route) -> bool {
    match &self.route {
      RootRoute::Main(base_route) => { other.eq(base_route) }
      RootRoute::BookViewer => { false }
      RootRoute::Setup => { false }
    }
  }
  pub fn set_setup_status(&mut self, status: bool, db: &DB) -> Result<()> {
    match &self.setup_is_done.eq(&status) {
      true => {}
      false => {
        update_table(db, Some(self.clone()), |new_settings| new_settings.setup_is_done = status)?;
        self.setup_is_done = status;
      }
    }
    Ok(())
  }
  pub fn get_self(db: &DB) -> Result<Self> {
    Ok(crud::get_primary::<Self>(1, db)?.unwrap())
  }
}
impl DefaultModel for Settings {
  fn default_model() -> Self
                     where Self: Sized + ToInput, {
    Self {
      id: 1,
      language: Lang::detect_system_lang(),
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
