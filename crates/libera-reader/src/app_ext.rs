use crate::app_dirs::AppDirs;
use crate::db::DB;
use crate::not_cached_books::NotCachedBooks;
use crate::services::Services;
use crate::settings::SETTINGS;
use gpui::App;

/// Extension trait providing convenient access to global services and state from `App` and `Context`.
pub trait AppExt {
  fn settings(&self) -> &SETTINGS;
  fn settings_mut(&mut self) -> &mut SETTINGS;
  fn db(&self) -> &DB;
  fn app_dirs(&self) -> &AppDirs;
  fn services(&self) -> &Services;
  fn services_mut(&mut self) -> &mut Services;
  fn not_cached_books(&self) -> &NotCachedBooks;
}

impl AppExt for App {
  #[inline(always)]
  fn settings(&self) -> &SETTINGS {
    self.global::<SETTINGS>()
  }

  #[inline(always)]
  fn settings_mut(&mut self) -> &mut SETTINGS {
    self.global_mut::<SETTINGS>()
  }

  #[inline(always)]
  fn db(&self) -> &DB {
    self.global::<DB>()
  }

  #[inline(always)]
  fn app_dirs(&self) -> &AppDirs {
    self.global::<AppDirs>()
  }

  #[inline(always)]
  fn services(&self) -> &Services {
    self.global::<Services>()
  }

  #[inline(always)]
  fn services_mut(&mut self) -> &mut Services {
    self.global_mut::<Services>()
  }

  #[inline(always)]
  fn not_cached_books(&self) -> &NotCachedBooks {
    self.global::<NotCachedBooks>()
  }
}
