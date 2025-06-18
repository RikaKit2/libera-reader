use crate::db::models::{ColorScheme, Theme};

impl Theme {
  pub fn make_sunset() -> Self {
    Self {
      name: "sunset".into(),
      color_scheme: ColorScheme::Dark,
      base_100: 0x121c22,
      base_200: 0x0e171e,
      base_300: 0x091319,
      base_color_content: 0x9fb9d0,
      primary_color: 0xff865b,
      primary_content_color: 0x160603,
      secondary_color: 0xfd6f9c,
      secondary_content_color: 0x160409,
      accent_color: 0xb387fa,
      accent_content_color: 0x0c0615,
      neutral_color: 0x1b262c,
      neutral_content_color: 0x94a0a9,
      info_color: 0x89e0eb,
      info_content_color: 0x071213,
      success_color: 0x071213,
      success_content_color: 0x0b120b,
      warning_color: 0xf1c892,
      warning_content_color: 0x140f08,
      error_color: 0x140f08,
      error_content_color: 0x140f08,
    }
  }
  pub fn make_wireframe()->Self{
    Self{
      name: "wireframe".into(),
      color_scheme: ColorScheme::Light,
      base_100: 0xffffff,
      base_200: 0xf5f5f5,
      base_300: 0xf5f5f5,
      base_color_content: 0x161616,
      primary_color: 0xd4d4d4,
      primary_content_color: 0x242424,
      secondary_color: 0xd4d4d4,
      secondary_content_color: 0xd4d4d4,
      accent_color: 0xd4d4d4,
      accent_content_color: 0xd4d4d4,
      neutral_color: 0xd4d4d4,
      neutral_content_color: 0x242424,
      info_color: 0x005889,
      info_content_color: 0xb8e6fe,
      success_color: 0x006044,
      success_content_color: 0xa3f2ce,
      warning_color: 0xa3f2ce,
      warning_content_color: 0xfde484,
      error_color: 0x9d0410,
      error_content_color: 0x9d0410,
    }
  }
}
