use std::sync::Arc;

use gxhash::HashSetExt;
use prisma_client_rust::QueryError;
use tokio::sync::RwLock;

use prisma::PrismaClient;
use prisma::settings;

use crate::db::prisma::prisma;
use crate::db::prisma::prisma::settings::Data;

pub struct SettingsManager {
  client: Arc<PrismaClient>,
  pub inn: Arc<RwLock<Data>>,
  pub target_ext: Arc<RwLock<gxhash::HashSet<String>>>,
}

impl SettingsManager {
  pub async fn new(client: Arc<PrismaClient>) -> Result<Self, QueryError> {
    match Self::get_instance(&client).await {
      Ok(res) => {
        match res {
          None => {
            match Self::create_instance(&client).await {
              Ok(inn) => {
                Ok(
                  Self {
                    client,
                    target_ext: Arc::from(RwLock::from(Self::get_target_extensions(&inn))),
                    inn: Arc::from(RwLock::from(inn)),
                  }
                )
              }
              Err(e) => {
                Err(e)
              }
            }
          }
          Some(inn) => {
            Ok(
              Self {
                client,
                target_ext: Arc::from(RwLock::from(Self::get_target_extensions(&inn))),
                inn: Arc::from(RwLock::from(inn)),
              }
            )
          }
        }
      }
      Err(e) => {
        Err(e)
      }
    }
  }

  pub async fn set_language(&mut self, lang: String) {
    let res = self.client.settings().update(
      settings::id::equals(1),
      vec![settings::language::set(lang)],
    ).exec().await.unwrap().language;
    self.inn.write().await.language = res;
  }
  pub async fn set_theme(&mut self, theme: String) {
    let res = self.client.settings().update(
      settings::id::equals(1),
      vec![settings::theme::set(theme)],
    ).exec().await.unwrap().theme;
    self.inn.write().await.theme = res;
  }
  pub async fn set_path_to_scan(&mut self, path: String) {
    let res = self.client.settings().update(
      settings::id::equals(1),
      vec![settings::path_to_scan::set(Option::from(path))],
    ).exec().await.unwrap().path_to_scan;
    self.inn.write().await.path_to_scan = res;
  }
  pub async fn ignore_pdf(&mut self, status: bool) {
    let res = self.client.settings().update(
      settings::id::equals(1),
      vec![settings::pdf::set(status)],
    ).exec().await.unwrap().pdf;
    self.inn.write().await.pdf = res;
    self.change_ext_usage_status_to_target_ext(status, "pdf".to_string()).await;
  }
  pub async fn ignore_epub(&mut self, status: bool) {
    let res = self.client.settings().update(
      settings::id::equals(1),
      vec![settings::epub::set(status)],
    ).exec().await.unwrap().epub;
    self.inn.write().await.epub = res;
    self.change_ext_usage_status_to_target_ext(status, "epub".to_string()).await;
  }
  pub async fn ignore_mobi(&mut self, status: bool) {
    let res = self.client.settings().update(
      settings::id::equals(1),
      vec![settings::mobi::set(status)],
    ).exec().await.unwrap().mobi;
    self.inn.write().await.mobi = res;
    self.change_ext_usage_status_to_target_ext(status, "mobi".to_string()).await;
  }
  pub async fn set_number_of_columns(&mut self, num: i32) {
    let res = self.client.settings().update(
      settings::id::equals(1),
      vec![settings::number_of_columns::set(num)],
    ).exec().await.unwrap().number_of_columns;
    self.inn.write().await.number_of_columns = res;
  }
  pub async fn set_page_scaling_factor(&mut self, scale: f64) {
    let res = self.client.settings().update(
      settings::id::equals(1),
      vec![settings::page_scaling_factor::set(scale)],
    ).exec().await.unwrap().page_scaling_factor;
    self.inn.write().await.page_scaling_factor = res;
  }
  pub async fn set_thumbnails_scaling_factor(&mut self, scale: f64) {
    let res = self.client.settings().update(
      settings::id::equals(1),
      vec![settings::thumbnails_scaling_factor::set(scale)],
    ).exec().await.unwrap().thumbnails_scaling_factor;
    self.inn.write().await.thumbnails_scaling_factor = res;
  }
  pub async fn set_reading_mode(&mut self, mode: String) {
    let res = self.client.settings().update(
      settings::id::equals(1),
      vec![settings::reading_mode::set(mode)],
    ).exec().await.unwrap().reading_mode;
    self.inn.write().await.reading_mode = res;
  }
  pub async fn set_workers_num(&mut self, num: i32) {
    let res = self.client.settings().update(
      settings::id::equals(1),
      vec![settings::workers_num::set(num)],
    ).exec().await.unwrap().workers_num;
    self.inn.write().await.workers_num = res;
  }
  fn get_target_extensions(inn: &Data) -> gxhash::HashSet<String> {
    let mut res = gxhash::HashSet::new();
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
  pub async fn get_instance(client: &PrismaClient) -> prisma_client_rust::Result<Option<Data>> {
    client.settings().find_first(
      vec![settings::id::equals(1)]
    ).exec().await
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
}

