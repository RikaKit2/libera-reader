use crate::app_dirs::AppDirs;
use crate::book_api::BookApi;
use crate::services::Services;
use crate::settings::Settings;


pub struct AppCore {
  pub app_dirs: AppDirs,
  pub services: Services,
  pub settings: Settings,
  pub book_api: BookApi,
}

impl AppCore {
  pub async fn new() -> Result<AppCore, Vec<String>> {
    match Settings::new() {
      Ok(settings) => {
        match AppDirs::new() {
          Ok(app_dirs) => {
            Ok(AppCore {
              app_dirs,
              services: Services::new(settings.get_path_to_scan_as_link().await, settings.get_target_ext(),
                                      settings.notify_receiver.clone()),
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
}
