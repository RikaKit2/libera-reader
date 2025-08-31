use anyhow::Result;
use libera_reader_core::ctx::Ctx;
use libera_reader_core::db::models::Book;
use mutool_bindings::{create_empty_book, download_mutool_if_missing_blocking, get_path_to_mutool};
use std::path::PathBuf;
use std::time::Duration;
use tokio::fs::{create_dir, remove_dir_all, rename};
use tokio::process::Command;
use tokio::time::sleep;
use tracing::{debug, error, info};

#[allow(dead_code)]
pub enum TestMode {
  Notify,
  PassiveScan,
}

const TIME_BETWEEN_TESTS: u64 = 300;
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
    let mut ctx = Ctx::new_for_test(tmp_dir.clone())?;
    ctx.settings.set_path_to_scan(tmp_dir.clone().to_string2())?;
    download_mutool_if_missing_blocking(&test_files).await?;
    Ok(Self {
      first_book: tmp_dir.join(&FIRST_BOOK),
      second_book: tmp_dir.join(&SECOND_BOOK),
      fist_dir: tmp_dir.join(&FIRST_DIR),
      second_dir: tmp_dir.join(&SECOND_DIR),
      test_files_dir: test_files,
      tmp_dir,
      test_mode,
      ctx,
    })
  }
  pub async fn create_first_book(&mut self) -> Result<()> {
    info!("Create first book");
    create_empty_book(&get_path_to_mutool(&self.test_files_dir), &self.first_book).await.unwrap();
    match self.test_mode {
      TestMode::Notify => {
        tokio::time::sleep(Duration::from_millis(TIME_BETWEEN_TESTS)).await;
      }
      TestMode::PassiveScan => {
        self.ctx.services.run_passive_scan().await?;
      }
    };
    self.test_fn(&self.first_book.to_string2(), |book: &Book| assert_eq!(&FIRST_BOOK, &book.book_name))?;
    Ok(())
  }
  pub async fn rename_first_book_to_second(&mut self) -> Result<()> {
    info!("File rename test: rename first book to second");
    match self.test_mode {
      TestMode::Notify => {
        let args = [&self.first_book.to_str().unwrap(), &self.second_book.to_str().unwrap()];
        assert!(Command::new("mv").args(args).spawn().is_ok());
        sleep(Duration::from_millis(TIME_BETWEEN_TESTS)).await;
      }
      TestMode::PassiveScan => {
        assert!(rename(&self.first_book, &self.second_book).await.is_ok());
        self.ctx.services.run_passive_scan().await?;
      }
    };
    self.test_fn(&self.second_book.to_string2(), |book: &Book| assert_eq!(&SECOND_BOOK, &book.book_name))?;
    Ok(())
  }
  pub async fn move_second_book_to_first_dir(&mut self) -> Result<()> {
    info!("File movement test: move second book to first dir");
    assert!(create_dir(&self.fist_dir).await.is_ok());
    let book_in_first_dir = self.tmp_dir.join(&FIRST_DIR).join(&SECOND_BOOK);
    match self.test_mode {
      TestMode::Notify => {
        let args = [&self.second_book.to_str().unwrap(), book_in_first_dir.parent().unwrap().to_str().unwrap()];
        assert!(Command::new("mv").args(args).spawn().is_ok());
        sleep(Duration::from_millis(TIME_BETWEEN_TESTS)).await;
      }
      TestMode::PassiveScan => {
        assert!(rename(&self.second_book, &book_in_first_dir).await.is_ok());
        self.ctx.services.run_passive_scan().await?;
      }
    }
    self.second_book = book_in_first_dir;
    self.test_fn(&self.second_book.to_string2(), |book: &Book| assert_eq!(&FIRST_DIR, &book.dir_name))?;
    Ok(())
  }
  pub async fn rename_first_dir_to_second(&mut self) -> Result<()> {
    info!("Dir renaming test: rename_first_dir_to_second");
    assert!(rename(&self.fist_dir, &self.second_dir).await.is_ok());
    match self.test_mode {
      TestMode::Notify => {
        sleep(Duration::from_millis(TIME_BETWEEN_TESTS)).await;
      }
      TestMode::PassiveScan => {
        self.ctx.services.run_passive_scan().await?;
      }
    }

    self.second_book = self.tmp_dir.join(&SECOND_DIR).join(&SECOND_BOOK);
    self.test_fn(&self.second_book.to_string2(), |book: &Book| assert_eq!(&SECOND_DIR, &book.dir_name))?;
    Ok(())
  }
  pub async fn rename_second_book_to_first_in_second_dir(&mut self) -> Result<()> {
    info!("File rename test2: rename second book to first in second dir");

    self.first_book = self.tmp_dir.join(&SECOND_DIR).join(&FIRST_BOOK);
    assert!(rename(&self.second_book, &self.first_book).await.is_ok());
    match self.test_mode {
      TestMode::Notify => {
        sleep(Duration::from_millis(TIME_BETWEEN_TESTS)).await;
      }
      TestMode::PassiveScan => {
        self.ctx.services.run_passive_scan().await?;
      }
    }

    self.test_fn(&self.first_book.to_string2(), |book: &Book| assert_eq!(&FIRST_BOOK, &book.book_name))?;
    Ok(())
  }
  pub async fn drop_second_dir(&mut self) -> Result<()> {
    info!("Dir deletion test: drop_second_dir");
    assert!(remove_dir_all(&self.second_dir).await.is_ok());
    match self.test_mode {
      TestMode::Notify => {
        sleep(Duration::from_millis(TIME_BETWEEN_TESTS)).await;
      }
      TestMode::PassiveScan => {
        self.ctx.services.run_passive_scan().await?;
      }
    }

    assert_eq!(Book::get_by_path(&self.first_book.to_string2(), &self.ctx.db)?, None, "there shouldn't be a book");
    Ok(())
  }
  pub async fn drop_files(tmp_dir: &PathBuf) {
    debug!("Drop test files");
    match remove_dir_all(tmp_dir).await {
      Ok(_) => {}
      Err(e) => error!("error when deleting tests_files_dir: {:?}", e),
    };
    match create_dir(tmp_dir).await {
      Ok(_) => {}
      Err(e) => error!("error when creating tests_files_dir: {:?}", e),
    };
  }
  fn test_fn<F>(&self, book_path_in_db: &String, assert_fn: F) -> Result<()>
  where
    F: Fn(&Book),
  {
    match Book::get_by_path(book_path_in_db, &self.ctx.db)? {
      None => {
        panic!("book in db not found: {:?}", book_path_in_db)
      }
      Some(book) => {
        assert_fn(&book);
      }
    }
    Ok(())
  }
  pub async fn run(&mut self) -> Result<()> {
    self.ctx.settings.set_path_to_scan(self.tmp_dir.to_string2())?;

    match self.test_mode {
      TestMode::Notify => {
        self.ctx.services.run_notify()?;
      }
      TestMode::PassiveScan => {}
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
pub trait EasyString {
  fn to_string2(&self) -> String;
}

impl EasyString for PathBuf {
  fn to_string2(&self) -> String {
    self.to_str().unwrap().to_string()
  }
}
impl Drop for TestLib {
  fn drop(&mut self) {
    self.ctx.services.stop().unwrap()
  }
}
