use anyhow::Result;
use std::fs;
use std::hash::{BuildHasher, Hasher};
use std::io::Read;
use std::path::{Path, PathBuf};
use tracing::Level;

pub enum FileSizeMeasure {
  MB, // Megabytes (1 MB = 1,048,576 bytes)
  B,  // Bytes
}
impl FileSizeMeasure {
  pub fn get_file_size(&self, path_to_file: &PathBuf, precision: u32) -> Result<f64> {
    match path_to_file.try_exists() {
      Ok(exists) => match exists {
        true => {
          let metadata = fs::metadata(path_to_file)?;
          let file_size_in_bytes = metadata.len();
          let file_size = match &self {
            FileSizeMeasure::MB => file_size_in_bytes as f64 / (1024 * 1024) as f64,
            FileSizeMeasure::B => file_size_in_bytes as f64,
          };
          let rounded_file_size = self.round_num(file_size, precision);
          Ok(rounded_file_size)
        }
        false => Err(std::io::Error::new(std::io::ErrorKind::NotFound, "File not found").into()),
      },
      Err(e) => Err(e.into()),
    }
  }
  fn round_num(&self, x: f64, decimals: u32) -> f64 {
    let y = 10i32.pow(decimals) as f64;
    (x * y).round() / y
  }
}

#[rustfmt::skip]
pub fn create_subscriber() ->Result<()> {
  let subscriber = tracing_subscriber::fmt()
    .pretty()
    .without_time()
    .compact()
    .with_file(false)
    .with_line_number(false)
    .with_thread_ids(false)
    .with_target(false)
    .with_max_level(Level::DEBUG)
    .finish();
  tracing::subscriber::set_global_default(subscriber)?;
  Ok(())
}

pub fn calc_file_hash<P: AsRef<Path>>(path_to_file: P) -> Result<String> {
  let mut hasher = gxhash::GxBuildHasher::default().build_hasher();
  let mut file = fs::File::open(path_to_file)?;
  loop {
    // Read the file in 1 MB chunks
    let mut buffer = [0; 1024 * 1024];
    let bytes_read = file.read(&mut buffer)?;
    if bytes_read == 0 {
      break;
    }
    hasher.write(&buffer[..bytes_read]);
  }
  Ok(data_encoding::HEXLOWER.encode(&hasher.finish().to_ne_bytes()))
}
