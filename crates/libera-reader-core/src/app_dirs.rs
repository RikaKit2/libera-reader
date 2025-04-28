use directories::ProjectDirs;
use std::path::PathBuf;
use tracing::debug;

pub struct AppDirs2 {
  pub inn: Option<AppDirs>,
}

impl AppDirs2 {
  pub fn new() -> Self {
    Self { inn: None }
  }
  pub fn set_base_dir(&mut self, data_dir: PathBuf) -> Result<(), Option<Vec<String>>> {
    match &self.inn {
      None => { Err(None) }
      Some(dirs) => {
        match &dirs.data_dir != &data_dir {
          true => { Err(None) }
          false => {
            match AppDirs::new(data_dir) {
              Ok(app_dirs) => {
                self.inn = Some(app_dirs);
                Ok(())
              }
              Err(e) => { Err(Some(e)) }
            }
          }
        }
      }
    }
  }
}
impl Default for AppDirs2 {
  fn default() -> Self {
    Self::new()
  }
}

pub struct AppDirs {
  pub data_dir: PathBuf,
  pub path_to_db: PathBuf,
  pub thumbnails_dir: PathBuf,
  pub dir_of_unhashed_books: PathBuf,
  pub dir_of_hashed_books: PathBuf,
  pub tts_models: PathBuf,
}

impl AppDirs {
  pub fn new(data_dir: PathBuf) -> Result<Self, Vec<String>> {
    match data_dir.exists() {
      true => {}
      false => {
        debug!("Creating data dir");
        std::fs::create_dir_all(&data_dir).unwrap();
      }
    }

    let path_to_db = data_dir.join("libera-reader").with_extension("redb");
    let thumbnails_dir = data_dir.join("thumbnails");
    let dir_of_unhashed_books = thumbnails_dir.join("unhashed_books");
    let dir_of_hashed_books = thumbnails_dir.join("hashed_books");
    let tts_models = data_dir.join("tts_models");

    let necessary_dirs = vec![&data_dir, &tts_models, &thumbnails_dir, &dir_of_unhashed_books, &dir_of_hashed_books];
    let poss_errors = Self::create_necessary_dirs(necessary_dirs);

    if poss_errors.len() > 0 {
      Err(poss_errors)
    } else {
      Ok(Self { data_dir, path_to_db, thumbnails_dir, dir_of_unhashed_books, dir_of_hashed_books, tts_models })
    }
  }

  pub fn set_base_dir(&mut self, target_dir: PathBuf) -> Result<(), Vec<String>> {
    match Self::new(target_dir) {
      Ok(new_self) => {
        self.path_to_db = new_self.path_to_db;
        self.thumbnails_dir = new_self.thumbnails_dir;
        self.dir_of_unhashed_books = new_self.dir_of_unhashed_books;
        self.dir_of_hashed_books = new_self.dir_of_hashed_books;
        self.tts_models = new_self.tts_models;
        Ok(())
      }
      Err(e) => { Err(e) }
    }
  }
  fn create_necessary_dirs(necessary_dirs: Vec<&PathBuf>) -> Vec<String> {
    let mut poss_errors: Vec<String> = vec![];
    for necessary_dir in necessary_dirs {
      if !necessary_dir.exists() {
        match std::fs::create_dir_all(necessary_dir) {
          Ok(_) => {}
          Err(err) => {
            eprint!("\ndir: {:?}\nerror: {:?}", necessary_dir, &err);
            poss_errors.push(err.to_string());
          }
        }
      }
    }
    poss_errors
  }
}
impl Default for AppDirs {
  fn default() -> Self {
    let proj_dirs = ProjectDirs::from("com", "RikaKit", "libera-reader").unwrap();
    Self::new(proj_dirs.data_dir().to_path_buf()).unwrap()
  }
}