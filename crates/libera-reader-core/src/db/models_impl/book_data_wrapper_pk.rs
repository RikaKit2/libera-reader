use crate::db::crud;
use crate::models::{BookData, BookDataWrapperPK, DataOfHashedBook, DataOfUnhashedBook};


impl BookDataWrapperPK {
  pub(crate) fn update_book_data<F>(self, book_data_change_func: F)
  where F: FnOnce(&mut BookData) {
    match self {
      BookDataWrapperPK::UniqueSize(book_size) => {
        let wrapper = crud::get_primary::<DataOfUnhashedBook>(book_size).unwrap();
        let mut new_wrapper = wrapper.clone();
        book_data_change_func(&mut new_wrapper.book_data);
        crud::update(wrapper, new_wrapper).unwrap();
      }
      BookDataWrapperPK::RepeatingSize(book_hash) => {
        let wrapper = crud::get_primary::<DataOfHashedBook>(book_hash).unwrap();
        let mut new_wrapper = wrapper.clone();
        book_data_change_func(&mut new_wrapper.book_data);
        crud::update(wrapper, new_wrapper).unwrap();
      }
    }
  }
}
