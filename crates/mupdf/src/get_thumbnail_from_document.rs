use crate::utils::mupdf_err_to_string;
use mupdf_sys::{mupdf_get_thumbnail_from_document, mupdf_get_thumbnail_from_document2};
use std::ffi::CString;


pub fn get_thumbnail_from_document(book_path: &str, max_store_size: usize,
                                   page_num: i32, alpha: f32,
                                   zoom: f32, quality: i32, storage: &mut [u8]) -> Result<(), String> {
  let c_book_path = CString::new(book_path).unwrap();
  let mupdf_status = unsafe {
    mupdf_get_thumbnail_from_document(max_store_size, c_book_path.as_ptr(),
                                      page_num, alpha, zoom, quality, storage.as_mut_ptr())
  };
  match mupdf_status.flag {
    true => { Ok(()) }
    false => { Err(mupdf_err_to_string(mupdf_status)) }
  }
}

pub fn get_thumbnail_from_document2(book_path: &str, max_store_size: usize,
                                   page_num: i32, alpha: f32,
                                   zoom: f32, quality: i32, path_to_out: &str) -> Result<(), String> {
  let c_book_path = CString::new(book_path).unwrap();
  let c_path_to_out = CString::new(path_to_out).unwrap();
  let mupdf_status = unsafe {
    mupdf_get_thumbnail_from_document2(max_store_size, c_book_path.as_ptr(),
                                      page_num, alpha, zoom, quality, c_path_to_out.as_ptr())
  };
  match mupdf_status.flag {
    true => { Ok(()) }
    false => { Err(mupdf_err_to_string(mupdf_status)) }
  }
}
