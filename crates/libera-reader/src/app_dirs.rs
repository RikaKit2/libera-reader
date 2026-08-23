use crate::utils::{error, title};
use directories::ProjectDirs;
use std::io::Error;
use std::path::PathBuf;
use std::sync::{Arc, RwLock, RwLockReadGuard};

#[derive(Clone)]
pub struct AppDirs {
  inn: Arc<RwLock<Dirs>>,
}

impl gpui::Global for AppDirs {}

impl AppDirs {
  pub fn new(path_to_data_dir: PathBuf) -> Result<Self, Vec<Error>> {
    let dirs = Dirs::new(path_to_data_dir)?;
    Ok(Self { inn: Arc::new(RwLock::new(dirs)) })
  }
  pub fn read(&self) -> RwLockReadGuard<'_, Dirs> {
    self.inn.read().unwrap()
  }
  pub fn new_with_default_data_dir() -> Result<AppDirs, Vec<Error>> {
    let proj_dirs = ProjectDirs::from("com", "RikaKit", "libera-reader").unwrap();
    Self::new(proj_dirs.data_dir().to_path_buf())
  }
}
pub struct Dirs {
  pub data_dir: PathBuf,
  pub path_to_db: PathBuf,
  pub thumbnails_dir: PathBuf,
  pub dir_of_unhashed_books: PathBuf,
  pub dir_of_hashed_books: PathBuf,
  pub tts_models: PathBuf,
  pub mutool: PathBuf,
}
impl Dirs {
  pub fn new(data_dir: PathBuf) -> Result<Self, Vec<Error>> {
    match data_dir.exists() {
      true => {}
      false => {
        error!("DATA DIR NOT EXISTS!");
        title!("CREATING DATA DIR");
        std::fs::create_dir_all(&data_dir).unwrap();
      }
    }
    let path_to_db = data_dir.join("libera-reader").with_extension("redb");
    let thumbnails_dir = data_dir.join("thumbnails");
    let dir_of_unhashed_books = thumbnails_dir.join("unhashed_books");
    let dir_of_hashed_books = thumbnails_dir.join("hashed_books");
    let tts_models = data_dir.join("tts_models");

    let necessary_dirs =
      vec![&data_dir, &tts_models, &thumbnails_dir, &dir_of_unhashed_books, &dir_of_hashed_books];
    let poss_errors = Self::create_necessary_dirs(necessary_dirs);

    #[cfg(target_os = "windows")]
    let mutool = data_dir.join("mutool.exe");
    #[cfg(target_os = "linux")]
    let mutool = data_dir.join("mutool");

    if !poss_errors.is_empty() {
      Err(poss_errors)
    } else {
      Ok(Self {
        data_dir,
        path_to_db,
        thumbnails_dir,
        dir_of_unhashed_books,
        dir_of_hashed_books,
        tts_models,
        mutool,
      })
    }
  }
  fn create_necessary_dirs(necessary_dirs: Vec<&PathBuf>) -> Vec<Error> {
    let mut poss_errors = vec![];
    for necessary_dir in necessary_dirs {
      if !necessary_dir.exists() {
        match std::fs::create_dir_all(necessary_dir) {
          Ok(_) => {}
          Err(err) => {
            error!("\ndir: {:?}\nerror: {:?}", necessary_dir, &err);
            poss_errors.push(err);
          }
        }
      }
    }
    poss_errors
  }
}
