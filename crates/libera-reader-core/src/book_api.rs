pub struct BookApi {}

impl BookApi {
  pub fn new() -> Self { Self {} }
  pub async fn get_book_by_name(&self, book_name: String) {}
  pub async fn get_books_from_db(&self) {}
}

