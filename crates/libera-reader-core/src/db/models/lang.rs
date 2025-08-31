use serde::{Deserialize, Serialize};
use sys_locale::get_locale;

pub enum LibraryText {}
pub enum HistoryText {}
pub enum FavoriteText {}
pub enum BookMarkText {}
pub enum StatsText {}
pub enum SettingsText {
  UsedFormats,
}
pub enum SetupText {
  Title,
  NextBtn,
  SelectBtn,
  TargetPath,
  TargetDir,
}
pub enum BookViewerText {}
pub enum ComponentsText {
  SearchPlaceholder,
}
pub trait GetText {
  fn get(&self, lang: &Lang) -> &'static str;
}
impl GetText for SetupText {
  fn get(&self, lang: &Lang) -> &'static str {
    match lang {
      Lang::EN => match self {
        SetupText::Title => "Setup",
        SetupText::NextBtn => "Next",
        SetupText::SelectBtn => "Select",
        SetupText::TargetPath => "Selected path for scanning:",
        SetupText::TargetDir => "Please select the directory for scanning:",
      },
      Lang::RU => match self {
        SetupText::Title => "Предварительная настройка",
        SetupText::NextBtn => "Далее",
        SetupText::SelectBtn => "Выбрать",
        SetupText::TargetPath => "Выбранный путь для сканирования:",
        SetupText::TargetDir => "Выберете пожалуйста директорию для сканирования:",
      },
    }
  }
}
impl GetText for SettingsText {
  fn get(&self, lang: &Lang) -> &'static str {
    match lang {
      Lang::EN => match self {
        SettingsText::UsedFormats => "Used Formats",
      },
      Lang::RU => match self {
        SettingsText::UsedFormats => "Используемые форматы",
      },
    }
  }
}
impl GetText for ComponentsText {
  fn get(&self, lang: &Lang) -> &'static str {
    match lang {
      Lang::EN => match self {
        ComponentsText::SearchPlaceholder => "Search",
      },
      Lang::RU => match self {
        ComponentsText::SearchPlaceholder => "Поиск",
      },
    }
  }
}
#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub enum Lang {
  EN,
  RU,
}
impl Lang {
  pub fn all() -> &'static [Lang] {
    &[Lang::EN, Lang::RU]
  }
  pub fn detect_system_lang() -> Lang {
    let locale = get_locale().unwrap_or_else(|| String::from("en-US"));
    let lang_code = locale.split(&['-', '_'][..]).next().unwrap_or("en").to_lowercase();
    match lang_code.as_str() {
      "ru" => Lang::RU,
      "en" => Lang::EN,
      _ => Lang::EN,
    }
  }
  pub fn get<T: GetText>(&self, target_enum: T) -> &'static str {
    target_enum.get(self)
  }
}
