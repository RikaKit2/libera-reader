use tracing::info;

use crate::db::{DB, DBType, models::books::Books};

pub(crate) enum BooksLocation {
  Disk,
  DB(super::BooksFromDB),
  DiskAndDB(super::BooksFromDB),
  None,
}
impl BooksLocation {
  pub(crate) fn classify(db: &DB, disk_books_count: usize) -> anyhow::Result<Self> {
    Ok(match &db.db_type {
      DBType::InMemory(_) => {
        if disk_books_count > 0 {
          info!("number of books on disk: {:?}", &disk_books_count);
          info!("number of books in db: 0");
          info!("number of new books: {:?}", &disk_books_count);
          Self::Disk
        } else {
          info!("number of books on disk: 0");
          info!("number of books in db: 0");
          Self::None
        }
      }
      DBType::InFile(_) => {
        let (books_from_db, db_books_count) = Books::all(&db);

        if books_from_db.len() > 0 && disk_books_count > 0 {
          info!("number of books on disk: {:?}", &disk_books_count);
          info!("number of books in db: {:?}", &books_from_db.len());
          Self::DiskAndDB(books_from_db)
        } else if books_from_db.len() > 0 && disk_books_count == 0 {
          info!("number of books on disk: 0");
          info!("number of books in db: {:?}", &books_from_db.len());
          info!("number of outdated books: {:?}", &db_books_count);
          Self::DB(books_from_db)
        } else if books_from_db.len() == 0 && disk_books_count > 0 {
          info!("number of books on disk: {:?}", &disk_books_count);
          info!("number of books in db: 0");
          info!("number of new books: {:?}", &disk_books_count);
          Self::Disk
        } else if books_from_db.len() == 0 && disk_books_count == 0 {
          info!("number of books on disk: 0");
          info!("number of books in db: 0");
          Self::None
        } else {
          info!("number of books on disk: 0");
          info!("number of books in db: 0");
          Self::None
        }
      }
    })
  }
}
