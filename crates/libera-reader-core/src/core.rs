use crate::app_dirs::AppDirs;
use crate::book_api::BookApi;
use crate::services::Services;
use crate::settings::Settings;
use crate::vars::SHUTDOWN;
use std::sync::atomic::Ordering;

pub struct Core {
  pub app_dirs: AppDirs,
  pub services: Services,
  pub settings: Settings,
  pub book_api: BookApi,
}

impl Core {
  pub fn new() -> Result<Core, Vec<String>> {
    match Settings::new() {
      Ok(settings) => {
        match AppDirs::new() {
          Ok(app_dirs) => {
            Ok(Core {
              app_dirs,
              services: Services::new(),
              settings,
              book_api: BookApi {},
            })
          }
          Err(errors) => {
            Err(errors)
          }
        }
      }
      Err(e) => {
        Err(vec![e.to_string()])
      }
    }
  }
  pub fn stop_all_services(&self) {
    self.services.stop_notify();
    SHUTDOWN.swap(true, Ordering::Relaxed);
  }
}
impl Drop for Core {
  fn drop(&mut self) {
    self.stop_all_services();
  }
}