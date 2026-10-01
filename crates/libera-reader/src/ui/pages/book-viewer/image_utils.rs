use gpui::RenderImage;
use image::Frame;
use image::ImageReader;
use smallvec::smallvec;
use std::io::Cursor;
use std::os::raw::c_int;
use std::sync::Arc;

/// Decodes in-memory image bytes (WebP with zero-swap native BGRA, or fallback PNG) into a `gpui::RenderImage`.
pub fn decode_page_image_bytes(bytes: &[u8]) -> Option<Arc<RenderImage>> {
  let mut width: c_int = 0;
  let mut height: c_int = 0;

  // 1. Fast path: native zero-swap WebP BGRA decoding
  if unsafe { libwebp_sys2::WebPGetInfo(bytes.as_ptr(), bytes.len(), &mut width, &mut height) } != 0
    && width > 0
    && height > 0
  {
    let w = width as u32;
    let h = height as u32;
    let len = (w * h * 4) as usize;
    let stride = (w * 4) as c_int;
    let mut buffer = vec![0u8; len];

    let res = unsafe {
      libwebp_sys2::WebPDecodeBGRAInto(
        bytes.as_ptr(),
        bytes.len(),
        buffer.as_mut_ptr(),
        len,
        stride,
      )
    };

    if !res.is_null()
      && let Some(img_buffer) = image::RgbaImage::from_raw(w, h, buffer)
    {
      let img_frame = Frame::new(img_buffer);
      return Some(Arc::new(RenderImage::new(smallvec![img_frame])));
    }
  }

  // 2. Fallback path for PNG bytes
  let img = ImageReader::new(Cursor::new(bytes)).with_guessed_format().ok()?.decode().ok()?;
  let mut rgba_img = img.to_rgba8();

  for px in rgba_img.as_chunks_mut::<4>().0 {
    px.swap(0, 2);
  }

  let img_frame = Frame::new(rgba_img);
  Some(Arc::new(RenderImage::new(smallvec![img_frame])))
}
