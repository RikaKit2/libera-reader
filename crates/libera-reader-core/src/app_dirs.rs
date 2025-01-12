use directories::ProjectDirs;
use std::path::PathBuf;


pub struct AppDirs {
  pub path_to_db: PathBuf,
  pub thumbnails_dir: PathBuf,
  pub dir_of_unhashed_books: PathBuf,
  pub dir_of_hashed_books: PathBuf,
  pub tts_models: PathBuf,
}

impl AppDirs {
  pub fn new() -> Result<Self, Vec<String>> {
    let mut poss_errors: Vec<String> = vec![];
    let proj_dirs = ProjectDirs::from("com", "RikaKit", "libera-reader-egui").unwrap();
    let data_dir = proj_dirs.data_dir().to_path_buf();

    let path_to_db = data_dir.join("libera-reader-egui").with_extension("redb");
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
}
