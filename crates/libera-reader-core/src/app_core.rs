use std::collections::VecDeque;
use std::sync::Arc;

use crossbeam_channel::Receiver;
use notify::{Event, RecommendedWatcher};
use prisma_client_rust::NewClientError;
use tokio::sync::RwLock;
use tracing::info;

use crate::app_dirs::AppDirs;
use crate::book_manager::BookManager;
use crate::db::crud;
use crate::db::model::{book_item, PrismaClient};
use crate::services;
use crate::settings_manger::SettingsManager;
use crate::utils::NotCachedBook;

#[derive(Debug)]
pub enum AppCoreError {
  DirsCreationErr(Vec<std::io::Error>),
  PrismaErr(NewClientError),
}

pub struct AppCore {
  app_dirs: AppDirs,
  client: Arc<PrismaClient>,
  pub settings_manager: SettingsManager,
  notify_events: Arc<Receiver<notify::Result<Event>>>,
  watcher: Arc<RwLock<RecommendedWatcher>>,
  book_manager: Arc<RwLock<BookManager>>,
}

impl AppCore {
  pub async fn new(path_to_db: Option<String>) -> Result<Self, AppCoreError> {
    match path_to_db {
      None => {
        match AppDirs::new() {
          Ok(app_dirs) => {
            Self::get_self(app_dirs).await
          }
          Err(e) => {
            Err(AppCoreError::DirsCreationErr(e))
          }
        }
      }
      Some(path) => {
        let app_dirs = AppDirs {
          path_to_db: path,
          thumbnails_dir: "".to_string(),
          tts_models: "".to_string(),
        };
        Self::get_self(app_dirs).await
      }
    }
  }
  pub async fn run_services(&mut self) {
    self.run_dir_scan_service().await;
    self.run_notify_service().await;
    self.run_data_extraction_service().await;
  }

  pub async fn run_notify_service(&mut self) {
    match &self.settings_manager.path_to_scan {
      None => {}
      Some(_path_to_scan) => {
        tokio::spawn(
          services::notify_service::run(self.notify_events.clone(), self.book_manager.clone())
        );
      }
    }
  }
  pub async fn run_dir_scan_service(&mut self) {
    match &self.settings_manager.path_to_scan {
      None => {}
      Some(_path_to_scan) => {
        // services::dir_scan_service::run(path_to_scan, &self.settings_manager.target_ext,
        //                                 &self.not_cached_books, &self.client).await;
      }
    }
  }
  pub async fn run_data_extraction_service(&mut self) {
    match &self.settings_manager.path_to_scan {
      None => {}
      Some(_path_to_scan) => {}
    }
  }
  pub async fn get_book_by_name(&self, book_name: String) -> Option<book_item::Data> {
    crud::get_book_item_by_name(book_name, &self.client).await
  }
  pub async fn get_books_from_db(&self) -> Vec<book_item::Data> {
    crud::get_book_items_from_db(&self.client).await
  }
  async fn get_self(app_dirs: AppDirs) -> Result<AppCore, AppCoreError> {
    match PrismaClient::_builder().with_url("file:".to_string() + &app_dirs.path_to_db).build().await {
      Ok(client) => {
        info!("\n path to db: {:?}", &app_dirs.path_to_db);
        client._db_push().await.unwrap();
        let client = Arc::from(client);
        let (tx, notify_events) = crossbeam_channel::bounded(20_000);
        let watcher = Arc::from(RwLock::from(
          notify::recommended_watcher(move |res| {
            tx.send(res).unwrap();
          }).unwrap()));
        let settings_manager = SettingsManager::new(client.clone(), watcher.clone()).await.unwrap();
        let target_ext = settings_manager.target_ext.clone();
        Ok(AppCore {
          app_dirs,
          book_manager: Arc::from(RwLock::from(BookManager::new(target_ext, client.clone()))),
          client,
          watcher,
          settings_manager,
          notify_events: Arc::from(notify_events),
        })
      }
      Err(e) => {
        Err(AppCoreError::PrismaErr(e))
      }
    }
  }
}
