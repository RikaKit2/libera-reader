use super::{BooksState, TargetList};
use gpui::Context;
use libera_reader_core::ctx::Ctx;
use libera_reader_core::db::models::books::Books;
use libera_reader_core::types::{HashMap, LibraryEvent};

impl BooksState {
  pub(crate) fn apply_event(&mut self, event: LibraryEvent, cx: &mut Context<Self>) {
    let mut needs_rebuild = true;

    match event {
      LibraryEvent::ThumbnailExtracted(path) => {
        // Thumbnail is ready! Mark in cache so UI shows the cover image.
        self.cover_cache.borrow_mut().insert(path, true);
        needs_rebuild = false; // No sorting needed
      }
      LibraryEvent::BookAdded(book) => {
        // Send new book to extraction queue
        let ctx = Ctx::global(cx);
        let _ = ctx.not_cached_books.tx().send(book.book_path.clone());

        let path = book.book_path.clone();
        let dir_books = self.books_map.entry(path.parent_dir.clone()).or_insert_with(|| Books {
          parent_dir: path.parent_dir.clone(),
          storage: HashMap::default(),
        });
        dir_books.storage.insert(path.file_name(), book);
      }
      LibraryEvent::BooksBatchAdded(books) => {
        for book in books {
          let path = book.book_path.clone();
          let dir_books = self.books_map.entry(path.parent_dir.clone()).or_insert_with(|| Books {
            parent_dir: path.parent_dir.clone(),
            storage: HashMap::default(),
          });
          dir_books.storage.insert(path.file_name(), book);
        }
      }
      LibraryEvent::BookUpdated(book) => {
        let path = book.book_path.clone();
        if let Some(dir_books) = self.books_map.get_mut(&path.parent_dir) {
          dir_books.storage.insert(path.file_name(), book);
        }
      }
      LibraryEvent::BookRemoved(path) => {
        if let Some(dir_books) = self.books_map.get_mut(&path.parent_dir) {
          dir_books.storage.swap_remove(&path.file_name());
          if dir_books.storage.is_empty() {
            self.books_map.swap_remove(&path.parent_dir);
          }
        }
      }
      LibraryEvent::BookPathUpdated { old_path, new_path } => {
        if let Some(dir_books) = self.books_map.get_mut(&old_path.parent_dir)
          && let Some(mut book) = dir_books.storage.swap_remove(&old_path.file_name())
        {
          book.book_path = new_path.clone();
          let new_dir_books =
            self.books_map.entry(new_path.parent_dir.clone()).or_insert_with(|| Books {
              parent_dir: new_path.parent_dir.clone(),
              storage: HashMap::default(),
            });
          new_dir_books.storage.insert(new_path.file_name(), book);
        }
      }
      LibraryEvent::BookMarkAdded { .. }
      | LibraryEvent::BookMarkUpdated { .. }
      | LibraryEvent::BookMarkRemoved { .. } => {}
      LibraryEvent::DirRemoved(dir) => {
        self.books_map.swap_remove(&dir);
      }
    }

    if needs_rebuild {
      self.rebuild_and_sort(TargetList::Library);
      self.rebuild_and_sort(TargetList::Favorites);
      self.rebuild_and_sort(TargetList::History);
      self.rebuild_and_sort(TargetList::Bookmarks);
    }
  }
}
