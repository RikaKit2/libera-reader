use directories::ProjectDirs;
use std::env;
use std::path::PathBuf;


pub struct AppDirs {
  pub path_to_db: PathBuf,
  pub thumbnails_dir: PathBuf,
  pub dir_of_unhashed_books: PathBuf,
  pub dir_of_hashed_books: PathBuf,
  pub tts_models: PathBuf,
}

impl AppDirs {
  pub fn new(target_dir: Option<PathBuf>) -> Result<Self, Vec<String>> {
    let mut poss_errors: Vec<String> = vec![];
    let data_dir = match target_dir {
      None => {
        let proj_dirs = ProjectDirs::from("com", "RikaKit", "libera-reader").unwrap();
        match env::var("libera_reader_data_dir") {
          Ok(val) => PathBuf::from(val),
          Err(_e) => proj_dirs.data_dir().to_path_buf(),
        }
      }
      Some(res) => { res }
    };
    let path_to_db = data_dir.join("libera-reader").with_extension("redb");
    let thumbnails_dir = data_dir.join("thumbnails");
    let dir_of_unhashed_books = thumbnails_dir.join("unhashed_books");
    let dir_of_hashed_books = thumbnails_dir.join("hashed_books");
    let tts_models = data_dir.join("tts_models");

    let necessary_dirs = vec![&data_dir, &tts_models, &thumbnails_dir, &dir_of_unhashed_books, &dir_of_hashed_books];
    for necessary_dir in necessary_dirs {
      if !necessary_dir.exists() {
        match std::fs::create_dir(necessary_dir) {
          Ok(_) => {}
          Err(err) => {
            poss_errors.push(err.to_string());
          }
        }
      }
    }
    if poss_errors.len() > 0 {
      Err(poss_errors)
    } else {
      Ok(Self { path_to_db, thumbnails_dir, dir_of_unhashed_books, dir_of_hashed_books, tts_models })
    }
  }
  pub fn change_base_dir(&mut self, target_dir: Option<PathBuf>) -> Result<(), Vec<String>> {
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
}
impl Default for AppDirs {
  fn default() -> Self {
    Self::new(None).unwrap()
  }
}