use gpui::Hsla;

pub fn adjust_brightness(color: Hsla, factor: f32) -> Hsla {
  Hsla { h: color.h, s: color.s, l: (color.l * factor).clamp(0.0, 1.0), a: color.a }
}
