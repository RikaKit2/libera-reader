use crate::db::crud;
use crate::db::models::{BookData, BookDataWrapperPK, DataOfHashedBook, DataOfUnhashedBook};
use crate::types::DB;
use anyhow::Result;

impl BookDataWrapperPK {
  pub(crate) fn update_book_data<F>(self, book_data_change_func: F, db: &DB) -> Result<()>
                                    where F: FnOnce(&mut BookData) {
    match self {
      BookDataWrapperPK::UniqueSize(book_size) => {
        let wrapper = crud::get_primary::<DataOfUnhashedBook>(book_size, db)?.unwrap();
        let mut new_wrapper = wrapper.clone();
        book_data_change_func(&mut new_wrapper.book_data);
        crud::update(wrapper, new_wrapper, db)?;
      }
      BookDataWrapperPK::RepeatingSize(book_hash) => {
        let wrapper = crud::get_primary::<DataOfHashedBook>(book_hash, db)?.unwrap();
        let mut new_wrapper = wrapper.clone();
        book_data_change_func(&mut new_wrapper.book_data);
        crud::update(wrapper, new_wrapper, db)?;
      }
    }
    Ok(())
  }
}
