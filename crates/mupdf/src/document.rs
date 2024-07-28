use std::ffi::{CStr, CString};
use std::fmt;

use byte_unit::{Byte, rust_decimal::prelude::ToPrimitive, Unit};

use mupdf_sys::{fz_context, fz_document, fz_drop_context, fz_drop_document, fz_drop_outline,
                fz_is_external_link, fz_outline, fz_resolve_link, mupdf_doc_page_count,
                mupdf_load_outline, mupdf_load_page, mupdf_lookup_metadata, mupdf_new_context,
                mupdf_open_document};

use crate::outline::Outline;
use crate::page::Page;

pub struct Document {
  inner: *mut fz_document,
  ctx: *mut fz_context,
}

impl Document {
  pub fn open(path_to_book: &str, max_store_size_in_mb: u64) -> Result<Self, String> {
    let c_path_to_book = CString::new(path_to_book).unwrap();
    let max_store_in_bytes = Byte::
    from_u64_with_unit(max_store_size_in_mb, Unit::MB).unwrap().as_u64().to_usize().unwrap();
    unsafe {
      let mupdf_res = mupdf_new_context(max_store_in_bytes);
      if mupdf_res.status {
        let ctx: *mut fz_context = mupdf_res.value.ctx;
        let inner_res = mupdf_open_document(ctx, c_path_to_book.as_ptr());
        if inner_res.status {
          Ok(Document { inner: inner_res.value.doc, ctx })
        } else {
          Err(CStr::from_ptr(inner_res.value.err_msg).to_str().unwrap().to_string())
        }
      } else {
        Err(CStr::from_ptr(mupdf_res.value.err_msg).to_str().unwrap().to_string())
      }
    }
  }
  pub fn page_count(&self) -> Result<u32, String> {
    unsafe {
      let mupdf_res = mupdf_doc_page_count(self.ctx, self.inner);
      if mupdf_res.status {
        Ok(mupdf_res.value.count.to_u32().unwrap())
      } else {
        Err(CStr::from_ptr(mupdf_res.value.err_msg).to_str().unwrap().to_string())
      }
    }
  }
  pub fn load_page(&self, page_num: i32) -> Result<Page, String> {
    unsafe {
      let mupdf_res = mupdf_load_page(self.ctx, self.inner, page_num);
      if mupdf_res.status {
        Ok(Page::new(self.ctx, mupdf_res.value.page))
      } else {
        Err(CStr::from_ptr(mupdf_res.value.err_msg).to_str().unwrap().to_string())
      }
    }
  }

  pub fn metadata(&self, key: MetadataKey) -> Result<String, String> {
    let c_key = CString::new(key.to_string()).unwrap();
    unsafe {
      let mupdf_res = mupdf_lookup_metadata(self.ctx, self.inner, c_key.as_ptr());
      if mupdf_res.status {
        let data = mupdf_res.value.res;
        if data.is_null() {
          Ok(String::new())
        } else {
          Ok(CStr::from_ptr(data).to_str().unwrap().to_string())
        }
      } else {
        Err(CStr::from_ptr(mupdf_res.value.err_msg).to_str().unwrap().to_string())
      }
    }
  }
  unsafe fn walk_outlines(&self, outline: *mut fz_outline) -> Vec<Outline> {
    let mut outlines = Vec::new();
    let mut next = outline;
    while !next.is_null() {
      let mut x = 0.0;
      let mut y = 0.0;
      let mut page = None;
      let title = CStr::from_ptr((*next).title).to_string_lossy().into_owned();
      let uri = if !(*next).uri.is_null() {
        if fz_is_external_link(self.ctx, (*next).uri) > 0 {
          Some(CStr::from_ptr((*next).uri).to_string_lossy().into_owned())
        } else {
          page = Some(
            fz_resolve_link(self.ctx, self.inner, (*next).uri, &mut x, &mut y).page
              as u32,
          );
          None
        }
      } else {
        None
      };
      let down = if !(*next).down.is_null() {
        self.walk_outlines((*next).down)
      } else {
        Vec::new()
      };
      outlines.push(Outline {
        title,
        uri,
        page,
        down,
        x,
        y,
      });
      next = (*next).next;
    }
    outlines
  }
  pub fn outlines(&self) -> Result<Vec<Outline>, String> {
    unsafe {
      let mupdf_res = mupdf_load_outline(self.ctx, self.inner);
      if mupdf_res.status {
        let outline = mupdf_res.value.res;
        if outline.is_null() {
          return Ok(Vec::new());
        }
        let toc = self.walk_outlines(outline);
        fz_drop_outline(self.ctx, outline);
        Ok(toc)
      } else {
        Err(CStr::from_ptr(mupdf_res.value.err_msg).to_str().unwrap().to_string())
      }
    }
  }
}

impl Drop for Document {
  fn drop(&mut self) {
    if !self.inner.is_null() {
      unsafe {
        fz_drop_document(self.ctx, self.inner);
        fz_drop_context(self.ctx);
      }
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MetadataKey {
  Format,
  Encryption,
  Author,
  Title,
  Producer,
  Creator,
  CreationDate,
  ModDate,
  Subject,
  Keywords,
}

impl fmt::Display for MetadataKey {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:?}", self)
  }
}