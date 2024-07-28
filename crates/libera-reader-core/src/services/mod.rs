use std::collections::HashSet;
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::book_manager::BookManager;
use crate::db::model::PrismaClient;
use crate::settings_manger::SettingsManager;

pub mod dir_scan_service;
pub mod data_extraction_service;
pub mod notify_service;


pub struct Services<'a> {
  book_manager: Arc<RwLock<BookManager>>,
  settings_manager: &'a SettingsManager,
}

impl<'a> Services<'a> {
  pub fn new(client: Arc<PrismaClient>, target_ext: Arc<RwLock<HashSet<String>>>, settings_manager: &'a SettingsManager) -> Self {
    Self { book_manager: Arc::from(RwLock::from(BookManager::new(target_ext, client.clone()))), settings_manager }
  }
}
