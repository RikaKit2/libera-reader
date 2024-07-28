use std::ffi::{CStr, CString};

use mupdf_sys::{fz_context, fz_drop_pixmap, fz_pixmap, mupdf_save_pixmap_as_jpeg};

pub struct Pixmap {
  ctx: *mut fz_context,
  inner: *mut fz_pixmap,
}

impl Pixmap {
  pub fn new(ctx: *mut fz_context, pixmap: *mut fz_pixmap) -> Pixmap {
    Pixmap { ctx, inner: pixmap }
  }

  pub fn save_as_jpeg(&mut self, quality: i32, path_to_out: String) -> Option<String> {
    unsafe {
      let c_path = CString::new(path_to_out).unwrap();
      let mupdf_res = mupdf_save_pixmap_as_jpeg(self.ctx, self.inner, quality, c_path.as_ptr());
      if mupdf_res.status {
        Option::from(None)
      } else {
        Option::from(CStr::from_ptr(mupdf_res.err_msg).to_str().unwrap().to_string())
      }
    }
  }
}

impl Drop for Pixmap {
  fn drop(&mut self) {
    if !self.inner.is_null() {
      unsafe { fz_drop_pixmap(self.ctx, self.inner) };
    }
  }
}
