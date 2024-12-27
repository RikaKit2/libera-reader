use crate::vars::SETTINGS;
use tracing::{error, info};

mod dir_scan_service;
mod data_extraction_service;
mod notify_service;


pub struct Services {}

impl Services {
  pub async fn run(&mut self) {
    let start = std::time::Instant::now();
    self.run_dir_scan();
    self.run_notify().await;
    self.run_data_extraction();
    info!("All services execution time is: {:?}", start.elapsed());
  }

  pub async fn run_notify(&mut self) {
    match SETTINGS.read().unwrap().path_to_scan.is_some() {
      true => {
        match tokio::spawn(notify_service::run()).await {
          Ok(_) => {}
          Err(err) => {
            error!("{:?}", err.to_string())
          }
        }
      }
      false => {
        info!("path_to_scan is None");
      }
    }
  }

  pub fn run_dir_scan(&self) {
    match &SETTINGS.read().unwrap().path_to_scan {
      None => {}
      Some(path_to_scan) => {
        tokio::spawn(dir_scan_service::run(path_to_scan.clone()));
      }
    }
  }
  pub fn run_data_extraction(&self) {}
}
impl Default for Services {
  fn default() -> Self { Self {} }
}
