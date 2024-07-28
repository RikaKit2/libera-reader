use std::sync::Arc;

use crate::db::crud;
use crate::db::model::{book_item, PrismaClient};

pub struct BookApi {
  client: Arc<PrismaClient>,
}

impl BookApi {
  pub fn new(client: Arc<PrismaClient>) -> Self { Self { client } }
  pub async fn get_book_by_name(&self, book_name: String) -> Option<book_item::Data> {
    crud::get_book_item_by_name(book_name, &self.client).await
  }
  pub async fn get_books_from_db(&self) -> Vec<book_item::Data> {
    crud::get_book_items_from_db(&self.client).await
  }
}