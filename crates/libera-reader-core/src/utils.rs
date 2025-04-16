use crate::types::{BookHash, FileSize};
use gxhash::GxBuildHasher;
use std::fs;
use std::hash::{BuildHasher, Hasher};
use std::io::Read;
use std::path::PathBuf;


pub(crate) enum RayonTaskType {
  ImgExtract,
  HashCalc,
}

fn round_num(x: f64, decimals: u32) -> f64 {
  let y = 10i32.pow(decimals) as f64;
  (x * y).round() / y
}
pub(crate) fn calc_file_size_in_mb(path_to_file: &PathBuf) -> FileSize {
  let metadata = fs::metadata(path_to_file).unwrap();
  let size_mb = metadata.len() as f64 / (1024.0 * 1024.0);
  round_num(size_mb, 6).to_string()
}
pub(crate) fn calc_file_hash(path_to_file: &PathBuf) -> BookHash {
  let mut hasher = GxBuildHasher::default().build_hasher();
  let mut file = fs::File::open(path_to_file).unwrap();
  loop {
    // Read the file in 1 MB chunks
    let mut buffer = [0; 1024 * 1024];
    let bytes_read = file.read(&mut buffer).unwrap();
    if bytes_read == 0 {
      break;
    }
    hasher.write(&buffer[..bytes_read]);
  }
  data_encoding::HEXLOWER.encode(&hasher.finish().to_ne_bytes())
}
pub(crate) fn get_num_of_threads(rayon_task_type: RayonTaskType) -> usize {
  let num_of_cpus = num_cpus::get();
  let num_threads_for_task: usize;
  if num_of_cpus >= 6 {
    match rayon_task_type {
      RayonTaskType::ImgExtract => { num_threads_for_task = num_of_cpus - 2; }
      RayonTaskType::HashCalc => { num_threads_for_task = 2; }
    }
  } else {
    match rayon_task_type {
      RayonTaskType::ImgExtract => {
        if num_of_cpus == 1 {
          num_threads_for_task = 1;
        } else {
          num_threads_for_task = num_of_cpus - 1;
        }
      }
      RayonTaskType::HashCalc => { num_threads_for_task = 2; }
    }
  }
  num_threads_for_task
}
