pub mod lang;
pub mod route;
pub mod theme;

use std::path::PathBuf;

use crate::db::models::{GetOrCreate, settings::route::SetupRoute::Welcome};

fn default_cache_size() -> u32 {
  80
}
use native_db::*;
#[allow(unused_imports)]
use native_model::{Model, native_model};
use serde::{Deserialize, Serialize};

pub use {
  lang::Lang,
  route::{RootRoute, Route},
  theme::AppTheme,
};

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum CardDisplayMode {
  #[default]
  Compact,
  Detailed,
  List,
}

#[derive(Serialize, Deserialize, Clone)]
#[native_model(id = 4, version = 1)]
#[native_db]
pub struct Settings {
  #[primary_key]
  pub id: u32,
  pub language: Lang,
  pub path_to_scan: Option<PathBuf>,
  pub previous_path_to_scan: Option<PathBuf>,
  pub theme: AppTheme,

  pub number_of_columns: u32,
  pub page_scaling_factor: f64,
  pub thumbnails_scaling_factor: f64,
  pub ui_zoom: f64,
  pub workers_num: u32,
  /// Number of thumbnail images to keep in the LRU cache
  #[serde(default = "default_cache_size")]
  pub image_cache_size: u32,
  pub route: RootRoute,
  pub setup_is_done: bool,
  pub card_display_mode: CardDisplayMode,
}

impl Default for Settings {
  fn default() -> Self {
    Self {
      id: 1,
      language: Lang::detect_system_lang(),
      path_to_scan: None,
      previous_path_to_scan: None,
      theme: AppTheme::EverforestDark,

      number_of_columns: 6,
      page_scaling_factor: 1.0,
      thumbnails_scaling_factor: 4.0,
      ui_zoom: 1.0,
      workers_num: 2,
      image_cache_size: 80,
      route: RootRoute::Setup(Welcome),
      setup_is_done: false,
      card_display_mode: CardDisplayMode::Compact,
    }
  }
}

impl GetOrCreate for Settings {}
