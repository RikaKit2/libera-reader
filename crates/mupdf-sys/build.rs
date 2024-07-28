use std::{env, fs};
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;

fn fail_on_empty_directory(name: &str) {
  if fs::read_dir(name).unwrap().count() == 0 {
    println!("The `{}` directory is empty, did you forget to pull the submodules?", name);
    println!("Try `git submodule update --init --recursive`");
    panic!();
  }
}

fn build_libmupdf() {
  let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
  let build_dir = out_dir.join("build");
  std::fs::create_dir_all(&build_dir).unwrap();

  let current_dir = env::current_dir().unwrap();
  let mupdf_src_dir = current_dir.join("mupdf");

  let mut make_flags = vec![
    "libs".to_owned(),
    format!("OUT={}", build_dir.display()),
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
    .current_dir(&mupdf_src_dir)
    .stdout(Stdio::inherit())
    .stderr(Stdio::inherit())
    .output()
    .expect("make failed");
  if !output.status.success() {
    panic!("Build error, exit code {}", output.status.code().unwrap());
  }
  println!("cargo:rustc-link-search=native={}", build_dir.display());
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
  bindings.write_to_file(out_path.join("bindings.rs"))
    .expect("Couldn't write bindings!");
}

fn main() {
  fail_on_empty_directory("mupdf");
  println!("cargo:rerun-if-changed=wrapper.c");
  println!("cargo:rerun-if-changed=wrapper.h");

  build_libmupdf();

  let mut build = cc::Build::new();
  build.file("wrapper.c").include("./mupdf/include");
  build.compile("libmupdf-wrapper.a");

  generate_bindings();
}
