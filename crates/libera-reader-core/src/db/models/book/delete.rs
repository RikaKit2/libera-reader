use crate::db::models::{Book, BookDataPK, DataOfHashedBook, DataOfUnhashedBook, GetBookData};
use crate::types::DB;
use anyhow::Result;
use native_db::ToInput;

impl Book {
  pub(crate) fn remove(self, db: &DB) -> Result<()> {
    fn inn_delete<T: ToInput + GetBookData + Clone>(data: T, book: Book, db: &DB) -> Result<()> {
      let book_data = data.get_book_data();
      if book_data.favorite == true || book_data.in_history == true {
        let mut new_pk_data = data.clone();
        new_pk_data.get_book_data_mut().is_deleted = true;
        db.update::<T>(data, new_pk_data)?;
      } else {
        db.remove::<T>(data)?;
        db.remove::<Book>(book)?;
      }
      Ok(())
    }
    match &self.book_data_pk {
      BookDataPK::UniqueSize(book_size) => {
        let old_pk_data = db.get_primary::<DataOfUnhashedBook>(book_size.clone())?.unwrap();
        inn_delete(old_pk_data, self, db)?;
      }
      BookDataPK::Hashed(book_hash) => {
        let old_pk_data = db.get_primary::<DataOfHashedBook>(book_hash.clone())?.unwrap();
        if old_pk_data.book_data.books_pk.len() == 1 {
          inn_delete(old_pk_data, self, db)?;
        } else if old_pk_data.book_data.books_pk.len() > 1 {
          db.remove::<Book>(self)?;
        }
      }
    }
    Ok(())
  }
}