use anyhow::Result;
use gpui::*;
use rust_embed::RustEmbed;
use std::borrow::Cow;

#[derive(RustEmbed)]
#[folder = "./assets/icons"]
pub struct Assets;

impl AssetSource for Assets {
  fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
    let path = path.trim_start_matches('/');

    if let Some(file) = Self::get(path) {
      return Ok(Some(file.data));
    }

    if let Some(file) = gpui_component_assets::Assets::get(path) {
      return Ok(Some(file.data));
    }

    if (path == "check" || path == "icons/check")
      && let Some(file) = gpui_component_assets::Assets::get("icons/check.svg")
    {
      return Ok(Some(file.data));
    }

    Ok(None)
  }

  fn list(&self, path: &str) -> Result<Vec<SharedString>> {
    let path = path.trim_start_matches('/');
    let mut paths = crate::types::HashSet::default();

    for p in Self::iter() {
      if p.starts_with(path) {
        paths.insert(SharedString::from(p.to_string()));
      }
    }

    for p in gpui_component_assets::Assets::iter() {
      if p.starts_with(path) {
        paths.insert(SharedString::from(p.to_string()));
      }
    }

    Ok(paths.into_iter().collect())
  }
}
