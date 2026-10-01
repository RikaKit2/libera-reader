use crate::utils::{error, title};
use directories::ProjectDirs;
use std::io::Error;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::Arc;

pub const UNHASHED_BOOKS_DIR: &str = "unhashed_books";
pub const HASHED_BOOKS_DIR: &str = "hashed_books";

#[derive(Clone)]
pub struct AppDirs {
  inn: Arc<Dirs>,
}

impl gpui::Global for AppDirs {}

impl Deref for AppDirs {
  type Target = Dirs;

  #[inline(always)]
  fn deref(&self) -> &Self::Target {
    &self.inn
  }
}

impl AppDirs {
  pub fn new(path_to_data_dir: PathBuf) -> Result<Self, Vec<Error>> {
    let dirs = Dirs::new(path_to_data_dir)?;
    Ok(Self { inn: Arc::new(dirs) })
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
    let dir_of_unhashed_books = thumbnails_dir.join(UNHASHED_BOOKS_DIR);
    let dir_of_hashed_books = thumbnails_dir.join(HASHED_BOOKS_DIR);
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

  /// Returns the base directory for a specific book on disk.
  /// If hash is provided, uses `thumbnails/hashed_books/<hash>`.
  /// Otherwise uses `thumbnails/unhashed_books/<book_size>`.
  pub fn book_dir(&self, book_size: u64, hash_opt: Option<&str>) -> PathBuf {
    match hash_opt {
      Some(hash) if !hash.is_empty() => self.dir_of_hashed_books.join(hash),
      _ => self.dir_of_unhashed_books.join(book_size.to_string()),
    }
  }

  /// Path to the cover image of the book (defaults to `cover.jpg`).
  pub fn book_cover_path(&self, book_size: u64, hash_opt: Option<&str>) -> PathBuf {
    self.book_dir(book_size, hash_opt).join("cover.jpg")
  }

  /// Finds an existing valid cover image for the book across all supported formats:
  /// `cover.jpg`, `cover.jpeg`, `cover.webp`, `cover.png`, followed by legacy `<size|hash>.png` / `.jpg`.
  pub fn find_existing_book_cover(
    &self, book_size: u64, hash_opt: Option<&str>,
  ) -> Option<PathBuf> {
    let dir = self.book_dir(book_size, hash_opt);
    for name in ["cover.jpg", "cover.jpeg", "cover.webp", "cover.png"] {
      let candidate = dir.join(name);
      if candidate.exists() && std::fs::metadata(&candidate).map(|m| m.len() > 0).unwrap_or(false) {
        return Some(candidate);
      }
    }

    let legacy_candidates = match hash_opt {
      Some(hash) if !hash.is_empty() => vec![
        self.dir_of_hashed_books.join(format!("{}.jpg", hash)),
        self.dir_of_hashed_books.join(format!("{}.jpeg", hash)),
        self.dir_of_hashed_books.join(format!("{}.webp", hash)),
        self.dir_of_hashed_books.join(format!("{}.png", hash)),
      ],
      _ => vec![
        self.dir_of_unhashed_books.join(format!("{}.jpg", book_size)),
        self.dir_of_unhashed_books.join(format!("{}.jpeg", book_size)),
        self.dir_of_unhashed_books.join(format!("{}.webp", book_size)),
        self.dir_of_unhashed_books.join(format!("{}.png", book_size)),
      ],
    };

    legacy_candidates.into_iter().find(|candidate| {
      candidate.exists() && std::fs::metadata(candidate).map(|m| m.len() > 0).unwrap_or(false)
    })
  }

  /// Path to the legacy fallback cover path (e.g. `unhashed_books/<size>.png` or `hashed_books/<hash>.png`).
  pub fn legacy_book_cover_path(&self, book_size: u64, hash_opt: Option<&str>) -> PathBuf {
    match hash_opt {
      Some(hash) if !hash.is_empty() => self.dir_of_hashed_books.join(format!("{}.png", hash)),
      _ => self.dir_of_unhashed_books.join(format!("{}.png", book_size)),
    }
  }

  /// Path to the directory containing rendered pages for this book.
  pub fn book_pages_dir(&self, book_size: u64, hash_opt: Option<&str>) -> PathBuf {
    self.book_dir(book_size, hash_opt).join("pages")
  }

  /// Path to a specific rendered page PNG on disk (`pages/p{page}_{dpi}dpi.png`).
  pub fn book_page_path(
    &self, book_size: u64, hash_opt: Option<&str>, page: usize, dpi: u32,
  ) -> PathBuf {
    self.book_pages_dir(book_size, hash_opt).join(format!("p{}_{}dpi.png", page, dpi))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use tempfile::tempdir;

  #[test]
  fn test_book_paths_generation() {
    let tmp = tempdir().unwrap();
    let dirs = Dirs::new(tmp.path().to_path_buf()).unwrap();
    let app_dirs = AppDirs { inn: Arc::new(dirs) };

    // Hashed book
    let hash_dir = app_dirs.book_dir(1024, Some("a1b2c3d4"));
    assert!(hash_dir.ends_with("thumbnails/hashed_books/a1b2c3d4"));
    let cover_path = app_dirs.book_cover_path(1024, Some("a1b2c3d4"));
    assert!(cover_path.ends_with("thumbnails/hashed_books/a1b2c3d4/cover.jpg"));
    let page_path = app_dirs.book_page_path(1024, Some("a1b2c3d4"), 5, 150);
    assert!(page_path.ends_with("thumbnails/hashed_books/a1b2c3d4/pages/p5_150dpi.png"));

    // Unhashed book
    let unhash_dir = app_dirs.book_dir(2048, None);
    assert!(unhash_dir.ends_with("thumbnails/unhashed_books/2048"));
    let unhash_cover = app_dirs.book_cover_path(2048, None);
    assert!(unhash_cover.ends_with("thumbnails/unhashed_books/2048/cover.jpg"));
  }

  #[test]
  fn test_find_existing_book_cover() {
    let tmp = tempdir().unwrap();
    let dirs = Dirs::new(tmp.path().to_path_buf()).unwrap();
    let app_dirs = AppDirs { inn: Arc::new(dirs) };

    // None when no cover exists
    assert!(app_dirs.find_existing_book_cover(5000, None).is_none());

    // Create legacy cover.png
    let legacy_dir = app_dirs.book_dir(5000, None);
    std::fs::create_dir_all(&legacy_dir).unwrap();
    let legacy_png = legacy_dir.join("cover.png");
    std::fs::write(&legacy_png, b"legacy png data").unwrap();
    assert_eq!(app_dirs.find_existing_book_cover(5000, None), Some(legacy_png.clone()));

    // Create modern cover.jpg - should take priority over legacy cover.png
    let modern_jpg = legacy_dir.join("cover.jpg");
    std::fs::write(&modern_jpg, b"modern jpg data").unwrap();
    assert_eq!(app_dirs.find_existing_book_cover(5000, None), Some(modern_jpg));
  }
}
