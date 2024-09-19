use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::process::Stdio;
use std::{env, fs};
use zip::ZipArchive;

fn extract_from_zip_archive(path_to_file: &str) {
  let path_to_file = File::open(path_to_file).unwrap();
  let archive = ZipArchive::new(path_to_file);
  let out_dir = Path::new(".");
  archive
    .unwrap()
    .extract(&out_dir)
    .expect("Failed to extract archive");
}
fn download_mupdf(path_to_out_dir: &str) {
  let url = "https://github.com/ArtifexSoftware/mupdf/archive/refs/tags/1.24.9.zip";
  let mut response = reqwest::blocking::get(url).expect("request failed");
  let mut file = File::create(path_to_out_dir).expect("Failed to open file");
  response.copy_to(&mut file).unwrap();
}
fn get_mupdf_if_necessary() {
  match fs::read_dir("mupdf") {
    Ok(_) => {}
    Err(_) => {
      let mupdf_archive = "mupdf-1.24.9.zip";
      match File::open(mupdf_archive) {
        Ok(_) => {}
        Err(_) => {
          download_mupdf(mupdf_archive);
        }
      };
      extract_from_zip_archive(mupdf_archive);

      fs::rename("mupdf-1.24.9", "mupdf").unwrap();
      fs::remove_file(mupdf_archive).expect("Failed: delete mupdf tar archive");
    }
  }
}

fn build_libmupdf() {
  let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
  if !out_dir.exists() {
    fs::create_dir(&out_dir).unwrap();
  }

  let current_dir = env::current_dir().unwrap();
  let mupdf_dir = current_dir.join("mupdf");

  let mupdf_release_dir = mupdf_dir.join("build").join("release");
  let libmupdf_file = mupdf_release_dir.join("libmupdf.a").exists();
  let libmupdf_third_file = mupdf_release_dir.join("libmupdf-third.a").exists();
  let build_is_need = !libmupdf_file && !libmupdf_third_file;

  if build_is_need {
    let mut make_flags = vec![
      "libs".to_owned(),
      "HAVE_X11=no".to_owned(),
      "HAVE_GLUT=no".to_owned(),
      "HAVE_CURL=no".to_owned(),
    ];

    // Enable parallel compilation
    if let Ok(n) = std::thread::available_parallelism() {
      make_flags.push(format!("-j{}", n));
    }

    let output: std::process::Output = Command::new("make")
      .args(&make_flags)
      .current_dir(&mupdf_dir)
      .stdout(Stdio::inherit())
      .stderr(Stdio::inherit())
      .output()
      .expect("make failed");

    if !output.status.success() {
      panic!("Build error, exit code {}", output.status.code().unwrap());
    }
  }
  println!("cargo:rustc-link-search=native={}", mupdf_release_dir.display());
  println!("cargo:rustc-link-lib=static=mupdf");
  println!("cargo:rustc-link-lib=static=mupdf-third");
}

fn generate_bindings() {
  let bindings = bindgen::Builder::default()
    .clang_arg("-I./mupdf/include/mupdf")
    .header("wrapper.c")
    .header("wrapper.h")
    .allowlist_function("fz_.*")
    .allowlist_function("pdf_.*")
    .allowlist_function("ucdn_.*")
    .allowlist_function("Memento_.*")
    .allowlist_function("mupdf_.*")
    .allowlist_type("fz_.*")
    .allowlist_type("pdf_.*")
    .allowlist_var("fz_.*")
    .allowlist_var("FZ_.*")
    .allowlist_var("pdf_.*")
    .allowlist_var("PDF_.*")
    .allowlist_var("UCDN_.*")
    .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
    .size_t_is_usize(true)
    .generate()
    .expect("Unable to generate bindings");

  // Write the bindings to the $OUT_DIR/bindings.rs file.
  let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
  bindings
    .write_to_file(out_path.join("bindings.rs"))
    .expect("Couldn't write bindings!");
}

fn main() {
  get_mupdf_if_necessary();
  println!("cargo:rerun-if-changed=wrapper.c");
  println!("cargo:rerun-if-changed=wrapper.h");

  build_libmupdf();

  let mut build = cc::Build::new();
  build.file("wrapper.c").include("./mupdf/include");
  build.compile("libmupdf-wrapper.a");

  generate_bindings();
}
