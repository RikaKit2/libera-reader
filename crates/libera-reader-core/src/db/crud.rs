use crate::db::model::{book_data, book_item};
use crate::db::model::book_data::Data;
use crate::db::model::PrismaClient;

// book_manager
pub async fn create_book_item(path_to_book: String, book_data_id: i32, path_to_dir: String,
                              dir_name: String, file_name: String, client: &PrismaClient) {
  client.book_item().create_unchecked(
    path_to_book, book_data_id, path_to_dir, dir_name, file_name, vec![],
  ).exec().await.unwrap();
}

pub async fn create_book_data(book_hash: String, file_size: f64, ext: String, client: &PrismaClient) -> Data {
  client.book_data().create_unchecked(book_hash, vec![
    book_data::file_size::set(Option::from(file_size)),
    book_data::extension::set(Option::from(ext)),
  ]).exec().await.unwrap()
}

pub async fn get_book_data(book_hash: String, client: &PrismaClient) -> Option<Data> {
  client.book_data().find_first(
    vec![book_data::hash::equals(book_hash)]
  ).exec().await.unwrap()
}

pub async fn get_book_data_by_id(id: i32, client: &PrismaClient) -> Option<Data> {
  client.book_data().find_first(
    vec![book_data::id::equals(id)]
  ).exec().await.unwrap()
}

pub async fn get_book_item_by_path(path_to_book: String, client: &PrismaClient) -> Option<book_item::Data> {
  client.book_item().find_first(
    vec![book_item::path_to_book::equals(path_to_book)]
  ).exec().await.unwrap()
}

pub async fn get_book_item_by_name(book_name: String, client: &PrismaClient) -> Option<book_item::Data> {
  client.book_item().find_first(
    vec![book_item::file_name::equals(book_name)]
  ).exec().await.unwrap()
}

pub async fn get_num_links_per_book_data(book_data_id: i32, client: &PrismaClient) -> i64 {
  client.book_item().count(
    vec![book_item::book_data_id::equals(book_data_id)]
  ).exec().await.unwrap()
}

pub async fn delete_book_data(id: i32, client: &PrismaClient) {
  client.book_data().delete(book_data::id::equals(id)).exec().await.unwrap();
}

pub async fn delete_book_item(id: i32, client: &PrismaClient) {
  client.book_item().delete(book_item::id::equals(id)).exec().await.unwrap();
}

pub async fn change_path_and_dir(new_path_to_book: String, old_path_to_book: String,
                                 new_path_to_dir: String, new_dir_name: String,
                                 client: &PrismaClient) {
  client.book_item().update(
    book_item::path_to_book::equals(old_path_to_book),
    vec![
      book_item::path_to_book::set(new_path_to_book),
      book_item::path_to_dir::set(new_path_to_dir),
      book_item::dir_name::set(new_dir_name),
    ],
  ).exec().await.unwrap();
}

pub async fn change_path_and_book_name(new_path_to_book: String, old_path_to_book: String,
                                       new_book_name: String, client: &PrismaClient) {
  client.book_item().update(
    book_item::path_to_book::equals(old_path_to_book),
    vec![
      book_item::path_to_book::set(new_path_to_book),
      book_item::file_name::set(new_book_name),
    ],
  ).exec().await.unwrap();
}

pub async fn change_path_and_ext(new_path_to_book: String, old_path_to_book: String,
                                 new_ext: String, client: &PrismaClient) {
  let book_data_id = client.book_item().update(
    book_item::path_to_book::equals(old_path_to_book),
    vec![book_item::path_to_book::set(new_path_to_book)],
  ).exec().await.unwrap().book_data_id;
  client.book_data().update(
    book_data::id::equals(book_data_id),
    vec![book_data::extension::set(Option::from(new_ext))],
  ).exec().await.unwrap();
}

pub async fn get_books_contains_path_to_dir(old_path_to_dir: String, client: &PrismaClient) -> Vec<book_item::Data> {
  client.book_item().find_many(
    vec![book_item::path_to_book::contains(old_path_to_dir)]
  ).exec().await.unwrap()
}

pub async fn get_book_items_from_db(client: &PrismaClient) -> Vec<book_item::Data> {
  client.book_item().find_many(vec![]).exec().await.unwrap()
}

pub async fn get_books_paths_from_db(client: &PrismaClient) -> Vec<book_item::Data> {
  client.book_item().find_many(vec![]).exec().await.unwrap()
}

async fn mark_path_to_book_as_invalid(book_id: i32, client: &PrismaClient) {
  client.book_item().update(
    book_item::id::equals(book_id),
    vec![book_item::path_is_valid::set(false)],
  ).exec().await;
}

pub async fn del_book(book: book_item::Data, client: &PrismaClient) {
  let book_data = get_book_data_by_id(book.book_data_id, client).await.unwrap();
  if book_data.in_history.eq(&false) && book_data.favorite.eq(&false) {
    let num_links_per_book_data = get_num_links_per_book_data(book.book_data_id, client).await;

    if num_links_per_book_data == 1 {
      delete_book_item(book.id, client).await;
      delete_book_data(book.book_data_id, client).await;
    } else if num_links_per_book_data > 1 {
      delete_book_item(book.id, client).await;
    }
  } else if book_data.in_history.eq(&false) || book_data.favorite.eq(&false) {
    mark_path_to_book_as_invalid(book.id, client).await;
  } else if book_data.in_history.eq(&true) && book_data.favorite.eq(&true) {
    mark_path_to_book_as_invalid(book.id, client).await;
  }
}

// dir_scan_service
pub async fn del_outdated_books(books_paths_from_disk: Vec<String>, client: &PrismaClient){
  client.book_item().delete_many(vec![
    book_item::path_is_valid::equals(true),
    book_item::path_to_book::not_in_vec(books_paths_from_disk),
    book_item::book_data_link::is(vec![
      book_data::in_history::equals(false),
      book_data::favorite::equals(false),
    ]),
    book_item::book_data_link::is_not(vec![
      book_data::book_mark::some(vec![]),
    ]),
  ]).exec().await.unwrap();
  client.book_data().delete_many(vec![
    book_data::book_item::none(vec![]),
  ]).exec().await.unwrap();
}

