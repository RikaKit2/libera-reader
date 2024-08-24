use std::sync::Arc;

use prisma_client_rust::NewClientError;
use tokio::sync::RwLock;
use tracing::info;

use crate::app_dirs::AppDirs;
use crate::book_api::BookApi;
use crate::book_manager::BookManager;
use crate::db::model::PrismaClient;
use crate::services::Services;
use crate::settings::Settings;

#[derive(Debug)]
pub enum AppCoreError {
  DirsCreationErr(Vec<std::io::Error>),
  PrismaErr(NewClientError),
}

pub struct AppCore {
  app_dirs: AppDirs,
  pub services: Services,
  pub settings: Settings,
  pub book_api: BookApi,
}

impl AppCore {
  pub async fn new(path_to_db: Option<String>) -> Result<AppCore, AppCoreError> {
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
  async fn get_self(app_dirs: AppDirs) -> Result<AppCore, AppCoreError> {
    match PrismaClient::_builder().with_url("file:".to_string() + &app_dirs.path_to_db).build().await {
      Ok(db_client) => {
        info!("\n path to db: {:?}", &app_dirs.path_to_db);
        db_client._db_push().await.unwrap();
        let db_client = Arc::from(db_client);

        let (tx, notify_receiver) = crossbeam_channel::bounded(20_000);

        let watcher = Arc::from(RwLock::from(
          notify::recommended_watcher(move |res| { tx.send(res).unwrap(); }).unwrap()));

        let settings = Settings::new(db_client.clone(), watcher.clone()).await.unwrap();

        let book_manager = Arc::from(RwLock::from(
          BookManager::new(settings.target_ext.clone(), db_client.clone())
        ));
        let services = Services::new(
          settings.path_to_scan.clone(),
          book_manager,
          watcher,
          Arc::from(notify_receiver),
          settings.target_ext.clone(),
          db_client.clone(),
        );
        Ok(AppCore { app_dirs, book_api: BookApi::new(db_client), services, settings })
      }
      Err(err) => {
        Err(AppCoreError::PrismaErr(err))
      }
    }
  }
}
