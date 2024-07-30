use std::collections::HashSet;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::Arc;

use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use prisma_client_rust::QueryError;
use tokio::sync::RwLock;

use model::PrismaClient;
use model::settings;

use crate::db::model;
use crate::db::model::settings::Data;

pub struct Settings {
  client: Arc<PrismaClient>,
  pub(crate) path_to_scan: Rc<RwLock<Option<String>>>,
  pub(crate) target_ext: Arc<RwLock<HashSet<String>>>,
  watcher: Arc<RwLock<RecommendedWatcher>>,
}

impl Settings {
  pub async fn new(client: Arc<PrismaClient>, watcher: Arc<RwLock<RecommendedWatcher>>) -> Result<Self, QueryError> {
    match Self::get_instance(&client).await {
      Ok(res) => {
        match res {
          None => {
            match Self::create_instance(&client).await {
              Ok(inn) => {
                Ok(Self::get_self(client, inn, watcher))
              }
              Err(e) => {
                Err(e)
              }
            }
          }
          Some(inn) => {
            Ok(Self::get_self(client, inn, watcher))
          }
        }
      }
      Err(e) => {
        Err(e)
      }
    }
  }
  pub async fn set_path_to_scan(&mut self, path: String) {
    self.watcher.write().await.watch(path.as_ref(), RecursiveMode::Recursive).unwrap();
    self.path_to_scan.write().await.replace(path.clone());
    self.client.settings().update(
      settings::id::equals(1),
      vec![settings::path_to_scan::set(Option::from(path))],
    ).exec().await.unwrap();
  }
  pub async fn set_language(&mut self, lang: String) {
    self.client.settings().update(
      settings::id::equals(1), vec![settings::language::set(lang)],
    ).exec().await.unwrap();
  }
  pub async fn set_theme(&mut self, theme: String) {
    self.client.settings().update(
      settings::id::equals(1), vec![settings::theme::set(theme)],
    ).exec().await.unwrap();
  }
  pub async fn ignore_pdf(&mut self, status: bool) {
    self.client.settings().update(
      settings::id::equals(1), vec![settings::pdf::set(status)],
    ).exec().await.unwrap();
    self.change_ext_usage_status_to_target_ext(status, "pdf".to_string()).await;
  }
  pub async fn ignore_epub(&mut self, status: bool) {
    self.client.settings().update(
      settings::id::equals(1), vec![settings::epub::set(status)],
    ).exec().await.unwrap();
    self.change_ext_usage_status_to_target_ext(status, "epub".to_string()).await;
  }
  pub async fn ignore_mobi(&mut self, status: bool) {
    self.client.settings().update(
      settings::id::equals(1),
      vec![settings::mobi::set(status)],
    ).exec().await.unwrap();
    self.change_ext_usage_status_to_target_ext(status, "mobi".to_string()).await;
  }
  pub async fn set_number_of_columns(&mut self, num: i32) {
    self.client.settings().update(
      settings::id::equals(1),
      vec![settings::number_of_columns::set(num)],
    ).exec().await.unwrap();
  }
  pub async fn set_page_scaling_factor(&mut self, scale: f64) {
    self.client.settings().update(
      settings::id::equals(1),
      vec![settings::page_scaling_factor::set(scale)],
    ).exec().await.unwrap();
  }
  pub async fn set_thumbnails_scaling_factor(&mut self, scale: f64) {
    self.client.settings().update(
      settings::id::equals(1),
      vec![settings::thumbnails_scaling_factor::set(scale)],
    ).exec().await.unwrap();
  }
  pub async fn set_reading_mode(&mut self, mode: String) {
    self.client.settings().update(
      settings::id::equals(1),
      vec![settings::reading_mode::set(mode)],
    ).exec().await.unwrap();
  }
  pub async fn set_num_mupdf_workers(&mut self, num: i32) {
    self.client.settings().update(
      settings::id::equals(1),
      vec![settings::workers_num::set(num)],
    ).exec().await.unwrap();
  }
  async fn create_instance(client: &PrismaClient) -> prisma_client_rust::Result<Data> {
    client.settings().create(vec![]).exec().await
  }
  async fn change_ext_usage_status_to_target_ext(&mut self, status: bool, ext_name: String) {
    if status == false {
      self.target_ext.write().await.remove(&ext_name);
    } else {
      self.target_ext.write().await.insert(ext_name);
    }
  }
  fn get_target_extensions(inn: &Data) -> HashSet<String> {
    let mut res = HashSet::new();
    if inn.pdf {
      res.insert("pdf".to_string());
    }
    if inn.mobi {
      res.insert("mobi".to_string());
    }
    if inn.epub {
      res.insert("epub".to_string());
    }
    res
  }
  pub async fn get_inn(&mut self) -> prisma_client_rust::Result<Option<Data>> {
    Settings::get_instance(&self.client).await
  }
  async fn get_instance(client: &PrismaClient) -> prisma_client_rust::Result<Option<Data>> {
    client.settings().find_first(
      vec![settings::id::equals(1)]
    ).exec().await
  }
  pub async fn get_path_to_scan(&self) -> Option<String> {
    let res = self.path_to_scan.read().await.deref().clone();
    res
  }
  fn get_self(client: Arc<PrismaClient>, inn: Data, watcher: Arc<RwLock<RecommendedWatcher>>) -> Self {
    Self {
      client,
      watcher,
      target_ext: Arc::from(RwLock::from(Self::get_target_extensions(&inn))),
      path_to_scan: Rc::from(RwLock::from(inn.path_to_scan)),
    }
  }
}

