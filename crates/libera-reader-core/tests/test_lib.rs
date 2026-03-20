use anyhow::Result;
use libera_reader_core::ctx::Ctx;
use libera_reader_core::db::models::books::Books;
use libera_reader_core::db::models::books::book::{Book, BookDir, BookPath};
use mutool::{create_empty_book, download_mutool_if_missing_blocking, get_path_to_mutool};
use std::path::PathBuf;
use std::time::Duration;
use tokio::fs::{create_dir, remove_dir_all, rename};
use tokio::process::Command;
use tokio::time::sleep;
use utils::{debug, error, title};

#[allow(dead_code)]
pub enum TestMode {
  Notify,
  ScanService,
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
  pub async fn create_first_book(&mut self) -> Result<()> {
    title!("CREATE FIRST BOOK");
    create_empty_book(&get_path_to_mutool(&self.test_files_dir), &self.first_book).await.unwrap();
    match self.test_mode {
      TestMode::Notify => {
        tokio::time::sleep(Duration::from_millis(TIME_BETWEEN_TESTS)).await;
      }
      TestMode::ScanService => {
        self.ctx.services.scan_service.run().await?;
      }
    };
    self.test_fn(&self.first_book, |book: &Book| assert_eq!(&FIRST_BOOK, &book.book_path.name))?;
    Ok(())
  }
  pub async fn rename_first_book_to_second(&mut self) -> Result<()> {
    title!("FILE RENAME TEST: RENAME FIRST BOOK TO SECOND");
    match self.test_mode {
      TestMode::Notify => {
        let args = [&self.first_book.to_str().unwrap(), &self.second_book.to_str().unwrap()];
        assert!(Command::new("mv").args(args).spawn().is_ok());
        sleep(Duration::from_millis(TIME_BETWEEN_TESTS)).await;
      }
      TestMode::ScanService => {
        assert!(rename(&self.first_book, &self.second_book).await.is_ok());
        self.ctx.services.scan_service.run().await?;
      }
    };
    self.test_fn(&self.second_book, |book: &Book| assert_eq!(&SECOND_BOOK, &book.book_path.name))?;
    Ok(())
  }
  pub async fn move_second_book_to_first_dir(&mut self) -> Result<()> {
    title!("FILE MOVEMENT TEST: MOVE SECOND BOOK TO FIRST DIR");
    assert!(create_dir(&self.fist_dir).await.is_ok());
    let book_in_first_dir = self.tmp_dir.join(FIRST_DIR).join(SECOND_BOOK);
    match self.test_mode {
      TestMode::Notify => {
        let args = [self.second_book.to_str().unwrap(), book_in_first_dir.parent().unwrap().to_str().unwrap()];
        assert!(Command::new("mv").args(args).spawn().is_ok());
        sleep(Duration::from_millis(TIME_BETWEEN_TESTS)).await;
      }
      TestMode::ScanService => {
        assert!(rename(&self.second_book, &book_in_first_dir).await.is_ok());
        self.ctx.services.scan_service.run().await?;
      }
    }
    self.second_book = book_in_first_dir;
    self
      .test_fn(&self.second_book, |book: &Book| assert_eq!(&FIRST_DIR, &book.book_path.parent_dir.dir_name()))?;
    Ok(())
  }
  pub async fn rename_first_dir_to_second(&mut self) -> Result<()> {
    title!("DIR RENAMING TEST: RENAME FIRST DIR TO SECOND");
    assert!(rename(&self.fist_dir, &self.second_dir).await.is_ok());
    match self.test_mode {
      TestMode::Notify => {
        sleep(Duration::from_millis(TIME_BETWEEN_TESTS)).await;
      }
      TestMode::ScanService => {
        self.ctx.services.scan_service.run().await?;
      }
    }

    self.second_book = self.tmp_dir.join(SECOND_DIR).join(SECOND_BOOK);
    self
      .test_fn(&self.second_book, |book: &Book| assert_eq!(&SECOND_DIR, &book.book_path.parent_dir.dir_name()))?;
    Ok(())
  }
  pub async fn rename_second_book_to_first_in_second_dir(&mut self) -> Result<()> {
    title!("FILE RENAME TEST: RENAME SECOND BOOK TO FIRST IN SECOND DIR");

    self.first_book = self.tmp_dir.join(SECOND_DIR).join(FIRST_BOOK);
    assert!(rename(&self.second_book, &self.first_book).await.is_ok());
    match self.test_mode {
      TestMode::Notify => {
        sleep(Duration::from_millis(TIME_BETWEEN_TESTS)).await;
      }
      TestMode::ScanService => {
        self.ctx.services.scan_service.run().await?;
      }
    }

    self.test_fn(&self.first_book, |book: &Book| assert_eq!(&FIRST_BOOK, &book.book_path.name))?;
    Ok(())
  }
  pub async fn drop_second_dir(&mut self) -> Result<()> {
    title!("DIR DELETION TEST: DROP SECOND DIR");
    assert!(remove_dir_all(&self.second_dir).await.is_ok());
    match self.test_mode {
      TestMode::Notify => {
        sleep(Duration::from_millis(TIME_BETWEEN_TESTS)).await;
      }
      TestMode::ScanService => {
        self.ctx.services.scan_service.run().await?;
      }
    }
    let parent_dir = self.first_book.parent().unwrap().to_path_buf();
    let book_dir = BookDir::new(parent_dir);
    let target_book = self.ctx.db.rt(|r| Books::get_by_parent_dir(book_dir, r))?;
    assert_eq!(target_book, None, "there shouldn't be a book");
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
  fn test_fn<F>(&self, book_path_in_db: &PathBuf, assert_fn: F) -> Result<()>
  where
    F: Fn(&Book),
  {
    let book_path = BookPath::new(book_path_in_db);
    match book_path {
      Some(book_path) => {
        let book_result = self.ctx.db.rt(|r| Books::get_by_path(book_path, r));
        match book_result {
          Ok(Some(book)) => {
            assert_fn(&book);
          }
          Ok(None) => {
            error!("book in db not found: {:?}", book_path_in_db);
            let Ok((books_from_db, book_count)) = self.ctx.db.rt(|r| Ok(Books::all(r))) else {
              error!("failed to get books from db");
              return Ok(());
            };
            debug!("book count: {}", &book_count);
            for (_book_dir, books) in books_from_db {
              debug!("{:?}", books);
            }
          }
          Err(e) => {
            error!("error getting book from db: {:?}", e);
          }
        }
      }
      None => {
        error!("error creating book path from path: {:?}", book_path_in_db);
      }
    };
    Ok(())
  }
  pub async fn run(&mut self) -> Result<()> {
    let path_to_scan = self.tmp_dir.clone();
    self.ctx.settings.set_path_to_scan(path_to_scan)?;

    match self.test_mode {
      TestMode::Notify => {
        self.ctx.services.notify_service.run()?;
      }
      TestMode::ScanService => {}
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
