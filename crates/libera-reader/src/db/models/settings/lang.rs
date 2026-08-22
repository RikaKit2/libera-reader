use gpui::SharedString;
use gpui_component::select::SelectItem;
use rust_i18n::t;
use serde::{Deserialize, Serialize};
use sys_locale::get_locale;

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub enum Lang {
  EN,
  RU,
}
impl Lang {
  pub fn all() -> &'static [Lang] {
    &[Lang::EN, Lang::RU]
  }
  pub fn detect_system_lang() -> Lang {
    let locale = get_locale().unwrap_or_else(|| "en-US".to_owned());
    let lang_code = locale.split(&['-', '_'][..]).next().unwrap_or("en").to_lowercase();
    match lang_code.as_str() {
      "ru" => Lang::RU,
      "en" => Lang::EN,
      _ => Lang::EN,
    }
  }
  pub fn to_string(&self) -> &'static str {
    match self {
      Lang::EN => "en",
      Lang::RU => "ru",
    }
  }
}

impl SelectItem for Lang {
  type Value = Lang;

  fn title(&self) -> SharedString {
    match self {
      Lang::EN => t!("lang.en").into(),
      Lang::RU => t!("lang.ru").into(),
    }
  }

  fn value(&self) -> &Self::Value {
    self
  }
}
