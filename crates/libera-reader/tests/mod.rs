use anyhow::Result;
use libera_reader::ctx::Ctx;
use libera_reader::db::models::books::book::{Book, BookDir, BookPath};
use mutool::{create_empty_book, download_mutool_if_missing_blocking, get_path_to_mutool};
use std::path::PathBuf;
use std::time::Duration;
use tokio::fs::{create_dir, remove_dir_all, rename};
use utils::{debug, error, title};
#[allow(dead_code)]
pub enum TestMode {
  Notify,
  ScanService,
}

const TIME_BETWEEN_TESTS: u64 = 800;
const FIRST_BOOK: &str = "first_book.pdf";
pub const SECOND_BOOK: &str = "second_book.pdf";
const FIRST_DIR: &str = "first_dir";
const SECOND_DIR: &str = "second_dir";

pub struct TestLib {
  test_mode: TestMode,

  first_book: PathBuf,
  second_book: PathBuf,

  fist_dir: PathBuf,
  second_dir: PathBuf,

  test_files_dir: PathBuf,
  tmp_dir: PathBuf,

  ctx: Ctx,
}

impl TestLib {
  pub async fn new(test_mode: TestMode, tmp_dir_name: &str) -> Result<Self> {
    let proj_root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let test_files = proj_root_dir.join("test_files");
    let tmp_dir = test_files.join(tmp_dir_name);

    Self::drop_files(&tmp_dir).await;

    let mut ctx = Ctx::new_for_test(tmp_dir.clone());
    let path_to_scan = tmp_dir.clone();
    ctx.settings.set_path_to_scan(path_to_scan)?;

    download_mutool_if_missing_blocking(&test_files).await?;

    Ok(Self {
      first_book: tmp_dir.join(FIRST_BOOK),
      second_book: tmp_dir.join(SECOND_BOOK),
      fist_dir: tmp_dir.join(FIRST_DIR),
      second_dir: tmp_dir.join(SECOND_DIR),
      test_files_dir: test_files,
      tmp_dir,
      test_mode,
      ctx,
    })
  }

  async fn wait_for_sync(&mut self) -> Result<()> {
    match self.test_mode {
      TestMode::Notify => {
        tokio::time::sleep(Duration::from_millis(TIME_BETWEEN_TESTS)).await;
      }
      TestMode::ScanService => {
        self.ctx.services.scan_service.run().await?;
      }
    }
    Ok(())
  }

  pub async fn create_first_book(&mut self) -> Result<()> {
    title!("CREATE FIRST BOOK");
    let path_to_mutool = get_path_to_mutool(&self.test_files_dir);
    create_empty_book(&path_to_mutool, &self.first_book).await.unwrap();

    self.wait_for_sync().await?;
    self.test_fn(&self.first_book, |book: &Book| {
      assert_eq!(FIRST_BOOK, book.book_path.file_name().as_ref())
    })?;
    Ok(())
  }

  pub async fn rename_first_book_to_second(&mut self) -> Result<()> {
    title!("FILE RENAME TEST: RENAME FIRST BOOK TO SECOND");
    rename(&self.first_book, &self.second_book).await?;

    self.wait_for_sync().await?;
    self.test_fn(&self.second_book, |book: &Book| {
      assert_eq!(SECOND_BOOK, book.book_path.file_name().as_ref())
    })?;
    Ok(())
  }

  pub async fn move_second_book_to_first_dir(&mut self) -> Result<()> {
    title!("FILE MOVEMENT TEST: MOVE SECOND BOOK TO FIRST DIR");
    create_dir(&self.fist_dir).await?;

    if let TestMode::Notify = self.test_mode {
      tokio::time::sleep(Duration::from_millis(50)).await;
    }

    let book_in_first_dir = self.tmp_dir.join(FIRST_DIR).join(SECOND_BOOK);

    rename(&self.second_book, &book_in_first_dir).await?;

    self.wait_for_sync().await?;

    self.second_book = book_in_first_dir;
    self.test_fn(&self.second_book, |book: &Book| {
      assert_eq!(FIRST_DIR, book.book_path.parent_dir.dir_name().as_ref())
    })?;
    Ok(())
  }

  pub async fn rename_first_dir_to_second(&mut self) -> Result<()> {
    title!("DIR RENAMING TEST: RENAME FIRST DIR TO SECOND");
    rename(&self.fist_dir, &self.second_dir).await?;

    self.wait_for_sync().await?;

    self.second_book = self.tmp_dir.join(SECOND_DIR).join(SECOND_BOOK);
    self.test_fn(&self.second_book, |book: &Book| {
      assert_eq!(SECOND_DIR, book.book_path.parent_dir.dir_name().as_ref())
    })?;
    Ok(())
  }

  pub async fn rename_second_book_to_first_in_second_dir(&mut self) -> Result<()> {
    title!("FILE RENAME TEST: RENAME SECOND BOOK TO FIRST IN SECOND DIR");

    self.first_book = self.tmp_dir.join(SECOND_DIR).join(FIRST_BOOK);
    rename(&self.second_book, &self.first_book).await?;

    self.wait_for_sync().await?;

    self.test_fn(&self.first_book, |book: &Book| {
      assert_eq!(FIRST_BOOK, book.book_path.file_name().as_ref())
    })?;
    Ok(())
  }

  pub async fn drop_second_dir(&mut self) -> Result<()> {
    title!("DIR DELETION TEST: DROP SECOND DIR");
    remove_dir_all(&self.second_dir).await?;

    self.wait_for_sync().await?;

    let parent_dir = self.first_book.parent().unwrap().to_path_buf();
    let book_dir = BookDir::new(parent_dir);
    let target_books = self.ctx.db.scan_books_by_parent_dir(book_dir.full_path().as_ref())?;

    assert!(target_books.is_empty(), "Directory exists in DB but should be empty");

    Ok(())
  }

  pub async fn drop_files(tmp_dir: &PathBuf) {
    debug!("Drop test files");
    let _ = remove_dir_all(tmp_dir).await; // Ignore error if folder doesn't exist yet
    match create_dir(tmp_dir).await {
      Ok(_) => {}
      Err(e) => error!("error when creating tests_files_dir: {:?}", e),
    };
  }

  fn test_fn<F>(&self, book_path_in_db: &PathBuf, assert_fn: F) -> Result<()>
  where
    F: Fn(&Book),
  {
    let book_path = BookPath::new(book_path_in_db)
      .expect("Failed to create BookPath from file path (check if extension is valid)");

    let book_result = self.ctx.db.get_book(book_path.clone());

    match book_result {
      Ok(Some(book)) => {
        assert_fn(&book);
      }
      Ok(None) => {
        let Ok(books_from_db) = self.ctx.db.scan_all_books() else {
          panic!("Failed to get all books from db for debug dump");
        };
        let book_count = books_from_db.len();
        for book in books_from_db {
          debug!("{:?}", book);
        }
        panic!("Book not found in DB: {:?} (Total books in DB: {})", book_path_in_db, book_count);
      }
      Err(e) => {
        panic!("Error getting book from db: {:?}", e);
      }
    }
    Ok(())
  }

  pub async fn run(&mut self) -> Result<()> {
    let path_to_scan = self.tmp_dir.clone();
    self.ctx.settings.set_path_to_scan(path_to_scan)?;

    if let TestMode::Notify = self.test_mode {
      self.ctx.services.notify_service.run()?;
    }

    self.create_first_book().await?;
    self.rename_first_book_to_second().await?;
    self.move_second_book_to_first_dir().await?;
    self.rename_first_dir_to_second().await?;
    self.rename_second_book_to_first_in_second_dir().await?;
    self.drop_second_dir().await?;

    Self::drop_files(&self.tmp_dir).await;
    Ok(())
  }
}
