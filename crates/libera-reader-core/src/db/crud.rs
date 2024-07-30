use std::collections::HashSet;
use std::ops::Deref;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::RwLock;
use tracing::info;

use crate::db::model::{book_data, book_item};
use crate::db::model::book_data::Data;
use crate::db::model::PrismaClient;
use crate::utils::{BookData, calc_file_size_in_mb, calc_gxhash_of_file};

// book_manager
pub async fn create_book_item(path_to_book: String, book_data_id: i32, path_to_dir: String,
                              dir_name: String, book_name: String, ext: String, client: &PrismaClient) {
  client.book_item().create_unchecked(
    path_to_book, book_data_id, path_to_dir, dir_name, book_name, ext, vec![],
  ).exec().await.unwrap();
}

pub async fn create_book_data(book_hash: String, book_size: f64, client: &PrismaClient) -> prisma_client_rust::Result<Data> {
  client.book_data().create(book_hash, book_size, vec![]).exec().await
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
    vec![book_item::book_name::equals(book_name)]
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
                                       book_name: String, client: &PrismaClient) {
  client.book_item().update(
    book_item::path_to_book::equals(old_path_to_book),
    vec![
      book_item::path_to_book::set(new_path_to_book),
      book_item::book_name::set(book_name),
    ],
  ).exec().await.unwrap();
}

pub async fn change_path_and_ext(new_path_to_book: String, old_path_to_book: String,
                                 new_ext: String, client: &PrismaClient) {
  client.book_item().update(
    book_item::path_to_book::equals(old_path_to_book),
    vec![
      book_item::path_to_book::set(new_path_to_book),
      book_item::ext::set(new_ext),
    ],
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
pub async fn del_outdated_books(books_paths_from_disk: Vec<String>, client: &PrismaClient) {
  let t1 = Instant::now();
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
  info!("\n Duration of deletion outdated books, eq: {:?}", t1.elapsed());
}

pub async fn mark_paths_to_outdated_user_books_as_invalid(books_paths_from_disk: Vec<String>, client: &PrismaClient) {
  client.book_item().update_many(vec![
    book_item::path_is_valid::equals(true),
    book_item::path_to_book::not_in_vec(books_paths_from_disk),
    book_item::book_data_link::is(vec![
      book_data::in_history::equals(true),
      book_data::favorite::equals(true),
      book_data::book_mark::some(vec![]),
    ]),
  ], vec![book_item::path_is_valid::set(false)],
  ).exec().await.unwrap();
}

async fn get_book_data_id_upsert(book_hash: String, book_size: f64, client: &PrismaClient) -> i32 {
  let res = client.book_data().upsert(
    book_data::hash::equals(book_hash.clone()),
    book_data::create(book_hash, book_size, vec![]),
    vec![],
  ).exec().await.unwrap().id;
  res
}

pub async fn push_new_books_to_db(new_books_for_db: HashSet<BookData>, client: &PrismaClient) {
  if new_books_for_db.len() > 0 {
    let start = Instant::now();
    let mut data: Vec<(String, i32, String, String, String, String, Vec<book_item::SetParam>)> = vec![];
    let mut avg_time_of_calc_book_hashes: Vec<Duration> = vec![];
    let mut avg_time_of_calc_book_sizes: Vec<Duration> = vec![];
    let mut avg_time_getting_data_id: Vec<Duration> = vec![];

    for i in new_books_for_db {
      let t0 = Instant::now();

      let book_hash = calc_gxhash_of_file(&i.path_to_book);
      avg_time_of_calc_book_hashes.push(t0.elapsed());

      let t1 = Instant::now();
      let book_size = calc_file_size_in_mb(&i.path_to_book);
      avg_time_of_calc_book_sizes.push(t1.elapsed());

      let t2 = Instant::now();
      let book_data_id = get_book_data_id_upsert(book_hash, book_size, client).await;
      avg_time_getting_data_id.push(t2.elapsed());
      let item = (
        i.path_to_book,
        book_data_id,
        i.path_to_dir,
        i.dir_name,
        i.book_name,
        i.ext,
        vec![]
      );
      data.push(item);
    }
    let t1: Duration = avg_time_of_calc_book_hashes.iter().sum();
    let t2: Duration = avg_time_of_calc_book_sizes.iter().sum();
    let t3: Duration = avg_time_getting_data_id.iter().sum();
    info!("\n AVG duration calc book hash: {:?}", t1);
    info!("\n AVG duration calc book size: {:?}", t2);
    info!("\n AVG duration getting book_data_id: {:?}", t3);
    info!("\n Duration of prepare data for creation book_items: {:?}", start.elapsed());

    let t1 = Instant::now();
    client.book_item().create_many(data).exec().await.unwrap();
    info!("\n Duration creation book_items: {:?}", t1.elapsed());
  };
}

