use crate::db::models::{Lang, TextId};
use sys_locale::get_locale;

impl Lang {
  pub fn all() -> &'static [Lang] {
    &[Lang::EN, Lang::RU]
  }
  pub fn from_locale(locale: &str) -> Self {
    let lang_code = locale.split(&['-', '_'][..]).next().unwrap_or("en").to_lowercase();
    match lang_code.as_str() {
      "ru" => Lang::RU,
      "en" => Lang::EN,
      _ => Lang::EN,
    }
  }
  pub fn detect_system_lang() -> Lang {
    let locale = get_locale().unwrap_or_else(|| String::from("en-US"));
    Lang::from_locale(&locale)
  }

  pub fn translate(&self, id: TextId) -> &'static str {
    match self {
      Lang::EN => match id {
        TextId::SetupPageTitle => { "Preliminary setting" }
        TextId::MessageOfSelectingTargetDir => { "Please select the Directory for scanning:" }
        TextId::SetupPageSelectBtn => { "Select" }
        TextId::TargetPath => { "Selected path for scanning:" }
        TextId::SetupPageNextBtn => { "Next" }
      },
      Lang::RU => match id {
        TextId::SetupPageTitle => { "Предварительная настройка" }
        TextId::MessageOfSelectingTargetDir => { "Выберете пожалуйста директорию для сканирования:" }
        TextId::SetupPageSelectBtn => { "Выбрать" }
        TextId::TargetPath => { "Выбранный путь для сканирования:" }
        TextId::SetupPageNextBtn => { "Далее" }
      },
    }
  }
}
