use gpui::{App, SharedString};
use gpui_component::{Theme, ThemeRegistry};
use libera_reader_core::{ctx::Ctx, error_handler::ErrorType::Other};
use std::path::PathBuf;
use utils::error;

pub fn set_app_theme(cx: &mut App, theme: String) {
  let theme_name = SharedString::from(theme);
  match ThemeRegistry::global(cx).themes().get(&theme_name).cloned() {
    Some(theme_config) => {
      Theme::global_mut(cx).apply_config(&theme_config);
    }
    None => {
      let ctx = Ctx::global(cx);
      let msg = format!("Theme not faund: {:?}", theme_name);
      error!("{}", &msg);
      ctx.error_handler.report(msg, Other);
    }
  };
}

pub fn init_theme(themes_dir: PathBuf, cx: &mut App) {
  match ThemeRegistry::watch_dir(themes_dir, cx, move |cx| {
    let ctx = Ctx::global(cx);
    let theme = ctx.theme().to_string();
    set_app_theme(cx, theme);
  }) {
    Ok(_) => {}
    Err(err) => {
      let ctx = Ctx::global(cx);
      ctx.error_handler.report(err.to_string(), Other);
    }
  };
}
