use utils::debug;

use crate::db::{DB, models::books::Books};

pub(crate) enum BooksLocation {
  Disk,
  DB(super::BooksFromDB),
  DiskAndDB(super::BooksFromDB),
  None,
}
impl BooksLocation {
  pub(crate) fn classify(db: &DB, disk_books_count: usize) -> anyhow::Result<Self> {
    let (books_from_db, db_books_count) = Books::all(db);
    debug!("{} books on disk, {} books in db", disk_books_count, db_books_count);

    if db_books_count > 0 && disk_books_count > 0 {
      debug!("Number of books on disk: {:?}", &disk_books_count);
      debug!("Number of books in db: {:?}", &db_books_count);
      Ok(Self::DiskAndDB(books_from_db))
    } else if db_books_count > 0 && disk_books_count == 0 {
      debug!("Number of books on disk: 0");
      debug!("Number of books in db: {:?}", &db_books_count);
      debug!("Number of outdated books: {:?}", &db_books_count);
      Ok(Self::DB(books_from_db))
    } else if db_books_count == 0 && disk_books_count > 0 {
      debug!("Number of books on disk: {:?}", &disk_books_count);
      debug!("Number of books in db: 0");
      debug!("Number of new books: {:?}", &disk_books_count);
      Ok(Self::Disk)
    } else {
      debug!("Number of books on disk: 0");
      debug!("Number of books in db: 0");
      Ok(Self::None)
    }
  }
}
