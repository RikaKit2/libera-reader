use crate::utils::mupdf_err_to_string;
use mupdf_sys::{fz_context, fz_drop_pixmap, fz_pixmap, mupdf_get_pixmap_as_jpeg, mupdf_save_pixmap_as_jpeg};
use std::ffi::CString;


pub struct Pixmap {
  ctx: *mut fz_context,
  inner: *mut fz_pixmap,
}

impl Pixmap {
  pub fn new(ctx: *mut fz_context, pixmap: *mut fz_pixmap) -> Pixmap {
    Pixmap { ctx, inner: pixmap }
  }

  pub fn save_as_jpeg(&mut self, quality: i32, path_to_out: String) -> Result<(), String> {
    let mupdf_status = unsafe {
      let c_path = CString::new(path_to_out).unwrap();
      mupdf_save_pixmap_as_jpeg(self.ctx, self.inner, quality, c_path.as_ptr())
    };
    match mupdf_status.flag {
      true => { Ok(()) }
      false => { Err(mupdf_err_to_string(mupdf_status)) }
    }
  }

  pub fn save_as_jpeg_to_storage(&mut self, quality: i32, storage: &mut [u8]) -> Result<(), String> {
    let mupdf_status = unsafe {
      mupdf_get_pixmap_as_jpeg(self.ctx, self.inner, quality, storage.as_mut_ptr())
    };
    match mupdf_status.flag {
      true => { Ok(()) }
      false => { Err(mupdf_err_to_string(mupdf_status)) }
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
