use gpui::RenderImage;
use image::Frame;
use image::ImageReader;
use smallvec::smallvec;
use std::io::Cursor;
use std::sync::Arc;

/// Decodes in-memory PNG bytes into a `gpui::RenderImage` with BGRA channels.
pub fn decode_page_image_bytes(bytes: &[u8]) -> Option<Arc<RenderImage>> {
  let img = ImageReader::new(Cursor::new(bytes)).with_guessed_format().ok()?.decode().ok()?;

  let mut rgba_img = img.to_rgba8();

  // ⚡️ OPTIMIZATION: GPUI natively expects BGRA format.
  // Fast in-place swap of R and B channels.
  for px in rgba_img.chunks_exact_mut(4) {
    px.swap(0, 2);
  }

  let img_frame = Frame::new(rgba_img);
  Some(Arc::new(RenderImage::new(smallvec![img_frame])))
}
