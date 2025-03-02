use mupdf_sys::mupdf_status;
use std::ffi::CStr;


pub fn mupdf_err_to_string(mupdf_status: mupdf_status) -> String {
  let err_msg = unsafe {
    if mupdf_status.err_msg.is_null() {
      "Unknown error".to_string()
    } else {
      CStr::from_ptr(mupdf_status.err_msg).to_string_lossy().into_owned()
    }
  };
  err_msg
}
