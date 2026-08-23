use anyhow::Result;
use libera_reader::app_dirs::AppDirs;
use libera_reader::books_state::BooksState;
use libera_reader::db::DB;
use libera_reader::not_cached_books::NotCachedBooks;
use libera_reader::services::Services;
use libera_reader::settings::SETTINGS;
use mimalloc::MiMalloc;
use rfd::AsyncFileDialog;
use std::thread::sleep;
use std::time::Duration;

#[global_allocator]
static GLOBAL_ALLOCATOR: MiMalloc = MiMalloc;

#[tokio::main]
async fn main() -> Result<()> {
  better_panic::install();
  libera_reader::utils::create_subscriber()?;
  let app_dirs = AppDirs::new_with_default_data_dir().unwrap();
  let path_to_db = app_dirs.read().path_to_db.clone();
  let db = DB::new(path_to_db)?;
  let mut settings = SETTINGS::new(db.clone())?;
  let (not_cached_books, rx) = NotCachedBooks::channel();
  let books_state = BooksState::from_deps(app_dirs.read().thumbnails_dir.clone(), &db);
  let mut services = Services::from_deps(
    settings.clone(),
    db.clone(),
    not_cached_books,
    app_dirs.clone(),
    books_state,
    rx,
  )?;
  let path_to_scan_is_some = settings.read().path_to_scan.is_some();
  match path_to_scan_is_some {
    true => {
      services.run().await?;
    }
    false => {
      println!("Please input path to scan:");
      if let Some(folder) = AsyncFileDialog::new().pick_folder().await {
        let path = folder.path().to_path_buf();
        settings.set_path_to_scan(path)?;
        services.run().await?;
      }
    }
  }
  loop {
    sleep(Duration::from_secs(10));
  }
}
