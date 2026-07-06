use gpui::RenderImage;
use image::Frame;
use image::ImageReader;
use image::imageops::FilterType;
use smallvec::smallvec;
use std::io::BufReader;
use std::os::unix::io::AsRawFd;
use std::path::Path;
use std::sync::Arc;

pub const ROW_H: f32 = 160.0;
pub const THUMB_MAX_PX: u32 = 200;

pub fn load_thumbnail(path: &Path) -> Option<Arc<RenderImage>> {
  let file = std::fs::File::open(path).ok()?;
  let fd = file.as_raw_fd();

  let img = ImageReader::new(BufReader::new(&file)).with_guessed_format().ok()?.decode().ok()?;

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

  // Clear NixOS disk cache (Page Cache)
  unsafe {
    libc::posix_fadvise(fd, 0, 0, libc::POSIX_FADV_DONTNEED);
  }

  // Assemble raw frame and pack into RenderImage
  let frame = Frame::new(rgba_img);
  let render_image = Arc::new(RenderImage::new(smallvec![frame]));

  Some(render_image)
}
