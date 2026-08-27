use gpui::SharedString;
use gpui_component::select::SelectItem;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct TtsEngineConfig {
  pub id: SharedString,
  pub name: SharedString,
}

impl SelectItem for TtsEngineConfig {
  type Value = TtsEngineConfig;

  fn title(&self) -> SharedString {
    self.name.clone()
  }

  fn value(&self) -> &Self::Value {
    self
  }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct TtsVoiceConfig {
  pub id: SharedString,
  pub name: SharedString,
  pub language: SharedString,
  pub engine_id: SharedString,
}

impl SelectItem for TtsVoiceConfig {
  type Value = TtsVoiceConfig;

  fn title(&self) -> SharedString {
    self.name.clone()
  }

  fn value(&self) -> &Self::Value {
    self
  }
}

#[derive(Clone, Debug)]
pub struct TtsConfigProvider;

impl TtsConfigProvider {
  pub fn available_engines() -> Vec<TtsEngineConfig> {
    vec![
      TtsEngineConfig { id: "piper".into(), name: "Piper (Офлайн)".into() },
      TtsEngineConfig { id: "rhvoice".into(), name: "RHVoice".into() },
      TtsEngineConfig { id: "silero".into(), name: "Silero TTS".into() },
      TtsEngineConfig { id: "openai".into(), name: "OpenAI TTS".into() },
    ]
  }

  pub fn available_voices(engine_id: &str) -> Vec<TtsVoiceConfig> {
    match engine_id {
      "piper" => vec![
        TtsVoiceConfig {
          id: "ru_RU-irina-medium".into(),
          name: "Ирина (ru)".into(),
          language: "ru".into(),
          engine_id: "piper".into(),
        },
        TtsVoiceConfig {
          id: "ru_RU-denis-medium".into(),
          name: "Денис (ru)".into(),
          language: "ru".into(),
          engine_id: "piper".into(),
        },
        TtsVoiceConfig {
          id: "en_US-amy-medium".into(),
          name: "Amy (en)".into(),
          language: "en".into(),
          engine_id: "piper".into(),
        },
        TtsVoiceConfig {
          id: "en_US-ryan-medium".into(),
          name: "Ryan (en)".into(),
          language: "en".into(),
          engine_id: "piper".into(),
        },
      ],
      "rhvoice" => vec![
        TtsVoiceConfig {
          id: "aleksandr".into(),
          name: "Александр (ru)".into(),
          language: "ru".into(),
          engine_id: "rhvoice".into(),
        },
        TtsVoiceConfig {
          id: "elena".into(),
          name: "Елена (ru)".into(),
          language: "ru".into(),
          engine_id: "rhvoice".into(),
        },
      ],
      "silero" => vec![
        TtsVoiceConfig {
          id: "v3_1_ru_kseniya".into(),
          name: "Ксения (ru)".into(),
          language: "ru".into(),
          engine_id: "silero".into(),
        },
        TtsVoiceConfig {
          id: "v3_1_ru_baya".into(),
          name: "Бая (ru)".into(),
          language: "ru".into(),
          engine_id: "silero".into(),
        },
      ],
      _ => vec![
        TtsVoiceConfig {
          id: "alloy".into(),
          name: "Alloy (en)".into(),
          language: "en".into(),
          engine_id: "openai".into(),
        },
        TtsVoiceConfig {
          id: "nova".into(),
          name: "Nova (en)".into(),
          language: "en".into(),
          engine_id: "openai".into(),
        },
      ],
    }
  }
}
