use gpui::RenderImage;
use image::Frame;
use image::ImageReader;
use image::imageops::FilterType;
use smallvec::smallvec;
use std::io::BufReader;
use std::path::Path;
use std::sync::Arc;

pub const THUMB_MAX_PX: u32 = 200;

/// Decodes WebP bytes directly into native BGRA memory for GPUI,
/// with 0 CPU channel swaps and optional SIMD-accelerated scaling in LibWebP.
pub fn decode_webp_to_gpui(webp_bytes: &[u8]) -> Option<Arc<RenderImage>> {
  use std::os::raw::c_int;

  let mut width: c_int = 0;
  let mut height: c_int = 0;

  // 1. Read width and height from WebP header (fast, without decoding)
  if unsafe {
    libwebp_sys2::WebPGetInfo(webp_bytes.as_ptr(), webp_bytes.len(), &mut width, &mut height)
  } == 0
  {
    return None;
  }

  if width <= 0 || height <= 0 {
    return None;
  }

  let w = width as u32;
  let h = height as u32;

  // When dimensions fit within THUMB_MAX_PX, decode directly into BGRA buffer (0 copy, 0 swap)
  if w <= THUMB_MAX_PX && h <= THUMB_MAX_PX {
    let len = (w * h * 4) as usize;
    let stride = (w * 4) as c_int;
    let mut buffer = vec![0u8; len];

    let res = unsafe {
      libwebp_sys2::WebPDecodeBGRAInto(
        webp_bytes.as_ptr(),
        webp_bytes.len(),
        buffer.as_mut_ptr(),
        len,
        stride,
      )
    };

    if res.is_null() {
      return None;
    }

    let img_buffer = image::RgbaImage::from_raw(w, h, buffer)?;
    let img_frame = Frame::new(img_buffer);
    return Some(Arc::new(RenderImage::new(smallvec![img_frame])));
  }

  // If thumbnail exceeds THUMB_MAX_PX, decode with LibWebP scaling directly into BGRA
  unsafe {
    let mut config: libwebp_sys2::WebPDecoderConfig = std::mem::zeroed();
    if libwebp_sys2::WebPInitDecoderConfig(&mut config) == 0 {
      return None;
    }

    let scale = (THUMB_MAX_PX as f32 / (w.max(h) as f32)).min(1.0);
    let scaled_w = ((w as f32 * scale).round() as u32).max(1);
    let scaled_h = ((h as f32 * scale).round() as u32).max(1);

    config.options.use_scaling = 1;
    config.options.scaled_width = scaled_w as c_int;
    config.options.scaled_height = scaled_h as c_int;
    config.output.colorspace = libwebp_sys2::MODE_BGRA;

    let status = libwebp_sys2::WebPDecode(webp_bytes.as_ptr(), webp_bytes.len(), &mut config);

    if status != libwebp_sys2::VP8_STATUS_OK {
      libwebp_sys2::WebPFreeDecBuffer(&mut config.output);
      return None;
    }

    let u = config.output.u.RGBA;
    let len = (scaled_w * scaled_h * 4) as usize;
    let slice = std::slice::from_raw_parts(u.rgba, len);
    let buffer = slice.to_vec();
    libwebp_sys2::WebPFreeDecBuffer(&mut config.output);

    let img_buffer = image::RgbaImage::from_raw(scaled_w, scaled_h, buffer)?;
    let img_frame = Frame::new(img_buffer);
    Some(Arc::new(RenderImage::new(smallvec![img_frame])))
  }
}

pub fn load_thumbnail(img_path: &Path) -> Option<Arc<RenderImage>> {
  let is_webp = img_path
    .extension()
    .and_then(|ext| ext.to_str())
    .map(|ext| ext.eq_ignore_ascii_case("webp"))
    .unwrap_or(false);

  if is_webp {
    if let Ok(img_file) = std::fs::File::open(img_path) {
      evict_file_page_cache(&img_file);
    }
    if let Ok(webp_bytes) = std::fs::read(img_path)
      && let Some(render_img) = decode_webp_to_gpui(&webp_bytes)
    {
      return Some(render_img);
    }
  }

  // Fallback for non-WebP (JPEG/PNG) or legacy caches
  let img_file = match std::fs::File::open(img_path) {
    Ok(f) => f,
    Err(_) => return None,
  };
  let reader = match ImageReader::new(BufReader::new(&img_file)).with_guessed_format() {
    Ok(r) => r,
    Err(_) => {
      let _ = std::fs::remove_file(img_path);
      return None;
    }
  };
  let img = match reader.decode() {
    Ok(img) => img,
    Err(_) => {
      let _ = std::fs::remove_file(img_path);
      return None;
    }
  };
  let resized = if img.width() > THUMB_MAX_PX || img.height() > THUMB_MAX_PX {
    img.resize(THUMB_MAX_PX, THUMB_MAX_PX, FilterType::Triangle)
  } else {
    img
  };

  let mut rgba_img = resized.to_rgba8();

  // Swap R and B channels on CPU for fallback formats
  for px in rgba_img.as_chunks_mut::<4>().0 {
    px.swap(0, 2);
  }

  evict_file_page_cache(&img_file);

  let img_frame = Frame::new(rgba_img);
  let render_image = Arc::new(RenderImage::new(smallvec![img_frame]));

  Some(render_image)
}
/// Advise the operating system to drop the file's contents from the page cache.
/// This prevents bulk thumbnail loading from polluting system RAM.
/// Cross-platform implementation:
/// - Linux: `posix_fadvise(..., POSIX_FADV_DONTNEED)`
/// - macOS: `fcntl(..., F_NOCACHE, 1)`
/// - Windows / other: safe no-op.
#[inline]
pub fn evict_file_page_cache(file: &std::fs::File) {
  #[cfg(target_os = "linux")]
  {
    use std::os::unix::io::AsRawFd;
    unsafe {
      libc::posix_fadvise(file.as_raw_fd(), 0, 0, libc::POSIX_FADV_DONTNEED);
    }
  }

  #[cfg(target_os = "macos")]
  {
    use std::os::unix::io::AsRawFd;
    unsafe {
      libc::fcntl(file.as_raw_fd(), libc::F_NOCACHE, 1);
    }
  }

  #[cfg(not(any(target_os = "linux", target_os = "macos")))]
  {
    let _ = file;
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_decode_webp_to_gpui() {
    let rgba = vec![255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255];
    let encoder = webp::Encoder::from_rgba(&rgba, 2, 2);
    let webp_mem = encoder.encode(80.0);

    let render_img = decode_webp_to_gpui(&webp_mem);
    assert!(render_img.is_some());
  }
}
