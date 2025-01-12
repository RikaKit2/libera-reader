use crate::book_api::BookApi;
use crate::models::Settings;
use crate::services::Services;


pub struct Core {
  pub services: Services,
  pub settings: Settings,
  pub book_api: BookApi,
}

impl Core {
  pub fn new() -> Self {
    Self {
      services: Services::new(),
      settings: Settings::new(),
      book_api: BookApi {},
    }
  }
}