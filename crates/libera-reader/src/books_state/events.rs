use super::{BooksState, TargetList};
use gpui::Context;
use libera_reader_core::ctx::Ctx;
use libera_reader_core::types::LibraryEvent;

impl BooksState {
  pub(crate) fn apply_event(&mut self, event: LibraryEvent, cx: &mut Context<Self>) {
    let mut needs_rebuild = true;

    match event {
      LibraryEvent::ThumbnailExtracted(path) => {
        let id: gpui::SharedString = path.full_path_string();
        self.cover_cache.borrow_mut().insert(id.clone(), true);
        if let Some(light) = self.books_map.get_mut(&id) {
          light.has_thumbnail = true;
        }
        needs_rebuild = false; // No sorting needed
      }
      LibraryEvent::BookAdded(book) => {
        // Send new book to extraction queue
        let ctx = Ctx::global(cx);
        let _ = ctx.not_cached_books.tx().send(book.book_path.clone());

        self.upsert_book(&book);
      }
      LibraryEvent::BooksBatchAdded(books) => {
        for book in books {
          self.upsert_book(&book);
        }
      }
      LibraryEvent::BookUpdated(book) => {
        self.upsert_book(&book);
      }
      LibraryEvent::BookRemoved(path) => {
        let id: gpui::SharedString = path.full_path_string();
        self.books_map.remove(&id);
        self.cover_cache.borrow_mut().remove(&id);
      }
      LibraryEvent::BookPathUpdated { old_path, new_path } => {
        let old_id: gpui::SharedString = old_path.full_path_string();
        if let Some(light) = self.books_map.remove(&old_id) {
          // Update the id in the light book
          let mut updated = light;
          let new_id: gpui::SharedString = new_path.full_path_string();
          updated.id = new_id.clone();
          updated.parent_dir = new_path.parent_dir.full_path();
          updated.name = new_path.name.clone();
          self.upsert_light(new_id, updated);
        }
        self.cover_cache.borrow_mut().remove(&old_id);
      }
      LibraryEvent::BookMarkAdded { .. }
      | LibraryEvent::BookMarkUpdated { .. }
      | LibraryEvent::BookMarkRemoved { .. } => {}
      LibraryEvent::DirRemoved(dir) => {
        let dir_path = dir.full_path().to_string();
        // Remove all books with this parent_dir
        self.books_map.retain(|_id, light| light.parent_dir.as_ref() != dir_path);
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
