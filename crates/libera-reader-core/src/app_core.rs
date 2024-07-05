use std::collections::VecDeque;
use std::sync::Arc;

use prisma_client_rust::NewClientError;
use tokio::sync::RwLock;
use tracing::info;

use crate::app_dirs::AppDirs;
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
  not_cached_books: Arc<RwLock<VecDeque<NotCachedBook>>>,
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
    match &self.settings_manager.path_to_scan {
      None => {}
      Some(_path) => {
        self.settings_manager.start_notify_service();
        tokio::spawn(
          services::notify_service::run(self.settings_manager.target_ext.clone(),
                                        self.not_cached_books.clone(), self.client.clone(),
                                        self.settings_manager.notify_events.clone())
        );
      }
    }
  }
  pub async fn get_book(&self, path_to_book: String) -> Option<book_item::Data> {
    crud::get_book_item(path_to_book, &self.client).await
  }
  pub async fn get_books(&self) -> Vec<book_item::Data> {
    crud::get_books(&self.client).await
  }
  async fn get_self(app_dirs: AppDirs) -> Result<AppCore, AppCoreError> {
    match PrismaClient::_builder().with_url("file:".to_string() + &app_dirs.path_to_db).build().await {
      Ok(client) => {
        info!("\n path to db: {:?}", &app_dirs.path_to_db);
        client._db_push().await.unwrap();
        let client = Arc::from(client);
        Ok(AppCore {
          settings_manager: SettingsManager::new(client.clone()).await.unwrap(),
          client,
          app_dirs,
          not_cached_books: Arc::from(RwLock::from(VecDeque::new())),
        })
      }
      Err(e) => {
        Err(AppCoreError::PrismaErr(e))
      }
    }
  }
}
