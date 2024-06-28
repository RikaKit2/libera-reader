use std::collections::VecDeque;
use std::sync::Arc;

use directories::ProjectDirs;
use prisma_client_rust::NewClientError;
use tokio::sync::RwLock;

use prisma::PrismaClient;

use crate::db::prisma::prisma;
use crate::settings_manger::SettingsManager;

pub struct NotCachedBook {
  pub book_hash: String,
  pub path_to_book: String,
}

pub struct AppDirs {
  pub path_to_db: String,
  pub thumbnails_dir: String,
  pub tts_dir: String,
}

impl AppDirs {
  pub fn new() -> Result<Self, Vec<std::io::Error>> {
    let mut output = vec![];
    let proj_dirs = ProjectDirs::from("com", "RikaKit", "libera-reader").unwrap();
    let data_dir = proj_dirs.data_dir().to_path_buf();

    let tts_dir = data_dir.join("tts");
    let thumbnails_dir = data_dir.join("thumbnails");

    let path_to_db = data_dir.join("libera-reader").with_extension("db");

    let necessary_dirs = [&data_dir, &tts_dir, &thumbnails_dir];
    for necessary_dir in necessary_dirs {
      if !necessary_dir.exists() {
        match std::fs::create_dir(necessary_dir) {
          Ok(_) => {}
          Err(e) => {
            output.push(e);
          }
        }
      }
    }
    if output.len() > 0 {
      Err(output)
    } else {
      Ok(Self {
        path_to_db: path_to_db.to_str().unwrap().to_string(),
        thumbnails_dir: thumbnails_dir.to_str().unwrap().to_string(),
        tts_dir: tts_dir.to_str().unwrap().to_string(),
      })
    }
  }
}

pub enum AppStateError {
  DirsCreationErr(Vec<std::io::Error>),
  PrismaErr(NewClientError),
}

pub struct AppState {
  pub app_dirs: Arc<RwLock<AppDirs>>,
  pub client: Arc<PrismaClient>,
  pub settings_manager: SettingsManager,
  pub not_cached_books: Arc<RwLock<VecDeque<NotCachedBook>>>,
}

impl AppState {
  pub async fn new() -> Result<Self, AppStateError> {
    match AppDirs::new() {
      Ok(app_dirs) => {
        let client = PrismaClient::_builder()
          .with_url(app_dirs.path_to_db.clone()).build().await;
        match client {
          Ok(client) => {
            let client = Arc::from(client);
            Ok(Self {
              settings_manager: SettingsManager::new(client.clone()).await.unwrap(),
              client,
              app_dirs: Arc::from(RwLock::from(app_dirs)),
              not_cached_books: Arc::from(RwLock::from(VecDeque::new())),
            })
          }
          Err(e) => {
            Err(AppStateError::PrismaErr(e))
          }
        }
      }
      Err(e) => {
        Err(AppStateError::DirsCreationErr(e))
      }
    }
  }
}
