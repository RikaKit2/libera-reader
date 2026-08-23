use crate::db::DB;
use crate::db::models::{AppTheme, CardDisplayMode, GetOrCreate, Lang, RootRoute, Settings};
use anyhow::Result;
use gpui::SharedString;
use std::path::PathBuf;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

#[derive(Clone)]
pub struct SETTINGS {
  inn: Arc<RwLock<Settings>>,
  db: DB,
}

impl gpui::Global for SETTINGS {}

/// Apply current language from settings to the localization engine and refresh windows.
pub fn apply_language(cx: &mut gpui::App) {
  let lang = cx.global::<SETTINGS>().read().language.to_string();
  rust_i18n::set_locale(lang);
  cx.refresh_windows();
}

/// Adjust color lightness by a multiplication factor.
pub fn adjust_brightness(color: gpui::Hsla, factor: f32) -> gpui::Hsla {
  gpui::Hsla { h: color.h, s: color.s, l: (color.l * factor).clamp(0.0, 1.0), a: color.a }
}

pub fn set_app_theme(cx: &mut gpui::App, theme: String) {
  let theme_name = gpui::SharedString::from(theme);
  match gpui_component::ThemeRegistry::global(cx).themes().get(&theme_name).cloned() {
    Some(theme_config) => {
      gpui_component::Theme::global_mut(cx).apply_config(&theme_config);
    }
    None => {
      crate::utils::error!("Theme not found: {:?}", theme_name);
    }
  };
}

pub fn init_theme(themes_dir: PathBuf, cx: &mut gpui::App) {
  match gpui_component::ThemeRegistry::watch_dir(themes_dir, cx, move |cx| {
    let theme = cx.global::<SETTINGS>().read().theme.to_string();
    set_app_theme(cx, theme);
  }) {
    Ok(_) => {}
    Err(err) => {
      crate::utils::error!("Failed to watch themes directory: {:?}", err);
    }
  };
}
impl SETTINGS {
  pub fn new(db: DB) -> Result<Self> {
    Ok(Self { inn: Arc::new(RwLock::new(Settings::get_or_create(1u32, &db)?)), db })
  }
  pub fn read(&self) -> RwLockReadGuard<'_, Settings> {
    self.inn.read().unwrap()
  }
  fn write(&mut self) -> RwLockWriteGuard<'_, Settings> {
    self.inn.write().unwrap()
  }
  pub fn set_path_to_scan(&mut self, new_path: PathBuf) -> Result<()> {
    let path_to_scan = self.read().path_to_scan.clone();
    let old_model = self.read().clone();
    match path_to_scan {
      None => {
        self.write().path_to_scan = Some(new_path);
      }
      Some(previous_path_to_scan) => match previous_path_to_scan.eq(&new_path) {
        true => {}
        false => {
          let mut lock = self.write();
          lock.path_to_scan = Some(new_path);
          lock.previous_path_to_scan = Some(previous_path_to_scan);
        }
      },
    }
    self.db.update(old_model, self.read().clone())?;
    Ok(())
  }
  pub fn get_path_to_scan_str(&self) -> Option<SharedString> {
    let guard = self.read();
    let res = guard.path_to_scan.as_ref().map(|path| path.to_string_lossy().to_string());
    res.map(|string| string.into())
  }
  pub fn get_path_to_scan_if_exists(&self) -> Option<PathBuf> {
    let guard = self.read();
    match &guard.path_to_scan {
      None => None,
      Some(path_to_scan) => match path_to_scan.exists() {
        true => Some(path_to_scan.clone()),
        false => None,
      },
    }
  }
  pub fn set_language(&mut self, lang: Lang) -> Result<()> {
    let old_model = self.read().clone();
    match old_model.language.eq(&lang) {
      true => {}
      false => {
        self.write().language = lang;
        self.db.update(old_model, self.read().clone())?;
      }
    };
    Ok(())
  }
  pub fn set_theme(&mut self, new_theme: &AppTheme) -> Result<()> {
    let old_model = self.read().clone();
    if &old_model.theme != new_theme {
      self.write().theme = new_theme.clone();
      self.db.update(old_model, self.read().clone())?;
    }
    Ok(())
  }
  pub fn set_route(&mut self, new_route: RootRoute) -> Result<()> {
    let old_model = self.read().clone();
    match old_model.route.eq(&new_route) {
      true => {}
      false => {
        self.write().route = new_route;
        self.db.update(old_model, self.read().clone())?;
      }
    }
    Ok(())
  }
  pub fn set_setup_status(&mut self, status: bool) -> Result<()> {
    let old_model = self.read().clone();
    match old_model.setup_is_done.eq(&status) {
      true => {}
      false => {
        self.write().setup_is_done = status;
        self.db.update(old_model, self.read().clone())?;
      }
    }
    Ok(())
  }
  pub fn set_number_of_columns(&mut self, columns: u32) -> Result<()> {
    let old_model = self.read().clone();
    if old_model.number_of_columns != columns {
      self.write().number_of_columns = columns;
      self.db.update(old_model, self.read().clone())?;
    }
    Ok(())
  }
  pub fn to_next_setup_route(&mut self) {
    let current_route = self.read().route;
    if let RootRoute::Setup(old_route) = current_route
      && let Some(new_route) = old_route.next()
    {
      let old_model = self.read().clone();
      self.write().route = RootRoute::Setup(new_route);
      self.db.update(old_model, self.read().clone()).unwrap();
    }
  }
  pub fn set_workers_num(&mut self, workers: u32) -> Result<()> {
    let old_model = self.read().clone();
    if old_model.workers_num != workers {
      self.write().workers_num = workers;
      self.db.update(old_model, self.read().clone())?;
    }
    Ok(())
  }
  pub fn set_image_cache_size(&mut self, size: u32) -> Result<()> {
    let old_model = self.read().clone();
    if old_model.image_cache_size != size {
      self.write().image_cache_size = size;
      self.db.update(old_model, self.read().clone())?;
    }
    Ok(())
  }
  pub fn set_display_mode(&mut self, mode: CardDisplayMode) -> Result<()> {
    let old_model = self.read().clone();
    if old_model.card_display_mode != mode {
      self.write().card_display_mode = mode;
      self.db.update(old_model, self.read().clone())?;
    }
    Ok(())
  }
  pub fn to_previous_setup_route(&mut self) {
    let current_route = self.read().route;
    if let RootRoute::Setup(old_route) = current_route
      && let Some(new_route) = old_route.back()
    {
      let old_model = self.read().clone();
      self.write().route = RootRoute::Setup(new_route);
      self.db.update(old_model, self.read().clone()).unwrap();
    };
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::db::models::settings::AppTheme;
  use crate::db::models::settings::Lang;

  #[test]
  fn test_settings_persistence() -> Result<()> {
    let tmp_dir = tempfile::tempdir()?;
    let db_path = tmp_dir.path().join("test_settings.redb");
    let db = DB::new(db_path)?;

    let mut settings = SETTINGS::new(db.clone())?;
    assert!(!settings.read().setup_is_done);

    settings.set_setup_status(true)?;
    assert!(settings.read().setup_is_done);

    settings.set_language(Lang::RU)?;
    assert_eq!(settings.read().language, Lang::RU);

    settings.set_theme(&AppTheme::AyuDark)?;
    assert_eq!(settings.read().theme, AppTheme::AyuDark);

    settings.set_number_of_columns(5)?;
    assert_eq!(settings.read().number_of_columns, 5);

    // Re-open settings with new instance to verify db persistence
    let settings_reopened = SETTINGS::new(db)?;
    assert!(settings_reopened.read().setup_is_done);
    assert_eq!(settings_reopened.read().language, Lang::RU);
    assert_eq!(settings_reopened.read().theme, AppTheme::AyuDark);
    assert_eq!(settings_reopened.read().number_of_columns, 5);

    Ok(())
  }
}
