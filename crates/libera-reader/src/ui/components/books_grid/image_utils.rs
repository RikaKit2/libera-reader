use gpui::RenderImage;
use image::Frame;
use image::ImageReader;
use image::imageops::FilterType;
use smallvec::smallvec;
use std::io::BufReader;
use std::path::Path;
use std::sync::Arc;

pub const THUMB_MAX_PX: u32 = 200;

pub fn load_thumbnail(img_path: &Path) -> Option<Arc<RenderImage>> {
  let img_file = std::fs::File::open(img_path).ok()?;
  let img =
    ImageReader::new(BufReader::new(&img_file)).with_guessed_format().ok()?.decode().ok()?;

  let resized = if img.width() > THUMB_MAX_PX || img.height() > THUMB_MAX_PX {
    img.resize(THUMB_MAX_PX, THUMB_MAX_PX, FilterType::Triangle)
  } else {
    img
  };

  let mut rgba_img = resized.to_rgba8();

  // ⚡️ OPTIMIZATION: GPUI natively accepts BGRA format.
  // Fast swap R and B channels on CPU before sending.
  for px in rgba_img.chunks_exact_mut(4) {
    px.swap(0, 2);
  }

  // Drop file contents from OS page cache to prevent RAM bloat
  evict_file_page_cache(&img_file);

  // Assemble raw frame and pack into RenderImage
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
