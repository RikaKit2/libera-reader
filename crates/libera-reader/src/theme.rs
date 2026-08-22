use crate::settings::SETTINGS;
use gpui::{App, Hsla, SharedString};
use gpui_component::{Theme, ThemeRegistry};
use std::path::PathBuf;
use utils::error;

/// Adjust color lightness by a multiplication factor.
pub fn adjust_brightness(color: Hsla, factor: f32) -> Hsla {
  Hsla { h: color.h, s: color.s, l: (color.l * factor).clamp(0.0, 1.0), a: color.a }
}

pub fn set_app_theme(cx: &mut App, theme: String) {
  let theme_name = SharedString::from(theme);
  match ThemeRegistry::global(cx).themes().get(&theme_name).cloned() {
    Some(theme_config) => {
      Theme::global_mut(cx).apply_config(&theme_config);
    }
    None => {
      error!("Theme not found: {:?}", theme_name);
    }
  };
}

pub fn init_theme(themes_dir: PathBuf, cx: &mut App) {
  match ThemeRegistry::watch_dir(themes_dir, cx, move |cx| {
    let theme = cx.global::<SETTINGS>().read().theme.to_string();
    set_app_theme(cx, theme);
  }) {
    Ok(_) => {}
    Err(err) => {
      error!("Failed to watch themes directory: {:?}", err);
    }
  };
}
