use std::ffi::CStr;

use serde::{Deserialize, Serialize};

use mupdf_sys::{fz_context, fz_drop_page, fz_page, mupdf_page_to_pixmap, mupdf_stext_page_as_json_from_page};

use crate::pixmap::Pixmap;

pub struct Page {
  ctx: *mut fz_context,
  inner: *mut fz_page,
}

impl Page {
  pub fn new(ctx: *mut fz_context, page: *mut fz_page) -> Page {
    Page { ctx, inner: page }
  }
  pub fn to_pixmap(&self, zoom: f32) -> Result<Pixmap, String> {
    unsafe {
      let mupdf_result = mupdf_page_to_pixmap(self.ctx, self.inner, 0.0, zoom);
      if mupdf_result.status {
        let pixmap = mupdf_result.value.pix;
        Ok(Pixmap::new(self.ctx, pixmap))
      } else {
        Err(CStr::from_ptr(mupdf_result.value.err_msg).to_str().unwrap().to_string())
      }
    }
  }
  pub fn get_stext_as_json(&self, scale: f32) -> Result<String, String> {
    unsafe {
      let mupdf_result = mupdf_stext_page_as_json_from_page(self.ctx, self.inner, scale);
      if mupdf_result.status {
        Ok(CStr::from_ptr(mupdf_result.value.text).to_str().unwrap().to_string())
      } else {
        Err(CStr::from_ptr(mupdf_result.value.err_msg).to_str().unwrap().to_string())
      }
    }
  }

  pub fn get_stext(&self, scale: f32) -> Result<SText, String> {
    match self.get_stext_as_json(scale) {
      Ok(data) => {
        let res: SText = serde_json::from_str(&*data).unwrap();
        Ok(res)
      }
      Err(err) => {
        Err(err)
      }
    }
  }
}

impl Drop for Page {
  fn drop(&mut self) {
    if !self.inner.is_null() {
      unsafe {
        fz_drop_page(self.ctx, self.inner);
      }
    }
  }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Font {
  pub name: String,
  pub family: String,
  pub weight: String,
  pub style: String,
  pub size: u32,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct BBox {
  pub x: u32,
  pub y: u32,
  pub w: u32,
  pub h: u32,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Line {
  pub wmode: u32,
  pub bbox: BBox,
  pub font: Font,
  pub x: u32,
  pub y: u32,
  pub text: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Block {
  pub r#type: String,
  pub bbox: BBox,
  pub lines: Vec<Line>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SText {
  pub blocks: Vec<Block>,
}
