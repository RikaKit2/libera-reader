use crate::app_dirs::AppDirs;
use crate::book_api::BookApi;
use crate::services::Services;
use crate::settings::Settings;
use crate::utils::{BookSize, Hash};
use gxhash::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;

pub(crate) type BookSizeSet = Arc<RwLock<HashSet<BookSize>>>;
pub(crate) type BookHashSet = Arc<RwLock<HashSet<Hash>>>;


pub struct AppCore {
  pub app_dirs: AppDirs,
  pub services: Services,
  pub settings: Settings,
  pub book_api: BookApi,
  book_sizes: BookSizeSet,
  book_hashes: BookHashSet,
}

impl AppCore {
  pub fn new() -> Result<AppCore, Vec<String>> {
    let settings = Settings::new();
    let path_to_scan = settings.get_path_to_scan_as_link();
    let db_book_data: BookSizeSet = Default::default();
    let hashed_book_data: BookHashSet = Default::default();
    match AppDirs::new() {
      Ok(app_dirs) => {
        Ok(AppCore {
          app_dirs,
          services: Services::new(path_to_scan, db_book_data.clone(), hashed_book_data.clone()),
          settings,
          book_api: BookApi {},
          book_sizes: db_book_data,
          book_hashes: hashed_book_data,
        })
      }
      Err(errors) => {
        Err(errors)
      }
    }
  }
}
