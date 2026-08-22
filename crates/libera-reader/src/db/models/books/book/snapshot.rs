use serde::{Deserialize, Serialize};

/// Lightweight, UI-agnostic snapshot of a `Book`.
///
/// Services construct this lightweight snapshot instead of passing full `Book`
/// structs across thread boundaries. `Book` carries `Vec<BookMark>`, multiple `String`s,
/// and a nested `BookPath` with multiple `SharedString`s.
///
/// `BookSnapshot` carries exactly the fields the UI needs to render and sort,
/// nothing else. Construction happens on the service side right after the book is
/// read from or written to the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookSnapshot {
  /// Full path string, same as `Book::id` / `BookPath::full_path_string()`.
  pub id: String,
  pub name: String,
  pub ext: String,
  pub parent_dir: String,
  pub size: u64,
  pub last_opened: u64,
  pub is_favorite: bool,
  pub has_thumbnail: bool,
  pub deleted: bool,
  pub bookmark_count: usize,
}

impl BookSnapshot {
  /// Build a snapshot from a full `Book`.
  ///
  /// `has_thumbnail` is decided by the caller (currently by consulting
  /// `BookSizes` / `BookHashes` via the thumbnail-path resolver). The `Book`
  /// struct's own `has_thumbnail` field is being phased out because it can
  /// desync from the actual state of PNG files on disk.
  pub fn from_book(book: &super::Book, has_thumbnail: bool) -> Self {
    let super::BookSize::BYTES(size) = book.book_size;
    Self {
      id: book.id.clone(),
      name: book.book_path.name.as_ref().to_string(),
      ext: book.book_path.ext.to_string(),
      parent_dir: book.parent_dir.clone(),
      size,
      last_opened: book.user_data.last_opened,
      is_favorite: book.user_data.favorite,
      has_thumbnail,
      deleted: book.book_path.deleted,
      bookmark_count: book.bookmarks.len(),
    }
  }
}
