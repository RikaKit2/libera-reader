use crate::db::models::{lang::Lang, route::RootRoute, theme::Theme, DefaultModel, GetOrCreate};
use native_db::*;
#[allow(unused_imports)]
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
#[native_model(id = 1, version = 1)]
#[native_db]
pub struct SettingsModel {
  #[primary_key]
  pub id: u32,
  pub language: Lang,
  pub path_to_scan: Option<String>,
  pub old_path_to_scan: Option<String>,
  pub theme: Theme,
  pub pdf: bool,
  pub epub: bool,
  pub mobi: bool,
  pub number_of_columns: u32,
  pub page_scaling_factor: f64,
  pub thumbnails_scaling_factor: f64,
  pub workers_num: u32,
  pub route: RootRoute,
  pub setup_is_done: bool,
}
impl DefaultModel for SettingsModel {
  fn default_model() -> Self
                     where Self: Sized + ToInput, {
    Self {
      id: 1,
      language: Lang::detect_system_lang(),
      path_to_scan: None,
      old_path_to_scan: None,
      theme: Theme::Sunset,
      pdf: true,
      epub: false,
      mobi: false,
      number_of_columns: 6,
      page_scaling_factor: 1.0,
      thumbnails_scaling_factor: 4.0,
      workers_num: 2,
      route: RootRoute::Setup,
      setup_is_done: false,
    }
  }
}
impl GetOrCreate for SettingsModel {}
