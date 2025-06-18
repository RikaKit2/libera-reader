pub fn adjust_brightness(color: u32, factor: f32) -> u32 {
  let r = ((color >> 16) & 0xFF) as f32;
  let g = ((color >> 8) & 0xFF) as f32;
  let b = (color & 0xFF) as f32;

  let r = (r * factor).clamp(0.0, 255.0) as u32;
  let g = (g * factor).clamp(0.0, 255.0) as u32;
  let b = (b * factor).clamp(0.0, 255.0) as u32;

  (r << 16) | (g << 8) | b
}
