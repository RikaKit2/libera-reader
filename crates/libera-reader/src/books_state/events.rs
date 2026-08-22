use super::{BooksState, TargetList};
use crate::ctx::Ctx;
use crate::types::LibraryEvent;
use gpui::Context;

impl BooksState {
  pub(crate) fn apply_event(&mut self, event: LibraryEvent, cx: &mut Context<Self>) {
    let mut needs_rebuild = true;
    let mut thumbnails_dirty = false;

    match event {
      LibraryEvent::ThumbnailExtracted(path) => {
        let id: gpui::SharedString = path.full_path_string();
        if let Some(light) = self.books_map.get_mut(&id) {
          light.has_thumbnail = true;
        }
        // Drop the cached entry so the next read picks up the freshly-written PNG.
        // We intentionally don't trigger a thumbnail re-resolve storm here: the
        // grid already listens to BooksState changes and reads via `get_or_resolve`,
        // which will repopulate this entry on demand.
        self.thumbnails.mark_extracted(&id);
        thumbnails_dirty = true;
        needs_rebuild = false; // No sorting needed
      }
      LibraryEvent::BookAdded(snapshot) => {
        // Send new book to extraction queue
        let ctx = Ctx::global(cx);
        let _ =
          ctx.not_cached_books.tx().send(crate::books_state::snapshot_to_book_path(&snapshot));

        self.upsert_snapshot(&snapshot);
        thumbnails_dirty = true;
      }
      LibraryEvent::BooksBatchAdded(snapshots) => {
        for snapshot in snapshots {
          self.upsert_snapshot(&snapshot);
        }
        thumbnails_dirty = true;
      }
      LibraryEvent::BookUpdated(snapshot) => {
        self.upsert_snapshot(&snapshot);
        thumbnails_dirty = true;
      }
      LibraryEvent::BookRemoved(path) => {
        let id: gpui::SharedString = path.full_path_string();
        self.books_map.remove(&id);
        self.thumbnails.remove(&id);
      }
      LibraryEvent::BookPathUpdated { old_path, new_path } => {
        let old_id: gpui::SharedString = old_path.full_path_string();
        let new_id: gpui::SharedString = new_path.full_path_string();
        if let Some(light) = self.books_map.remove(&old_id) {
          let mut updated = light;
          updated.id = new_id.clone();
          updated.parent_dir = new_path.parent_dir.full_path();
          updated.name = new_path.name.clone();
          self.upsert_light(new_id.clone(), updated);
        }
        self.thumbnails.rename(&old_id, new_id);
      }
      LibraryEvent::BookMarkAdded { book_path } => {
        let id: gpui::SharedString = book_path.full_path_string();
        if let Some(light) = self.books_map.get_mut(&id) {
          light.bookmark_count += 1;
        }
        needs_rebuild = true;
      }
      LibraryEvent::BookMarkRemoved { book_path } => {
        let id: gpui::SharedString = book_path.full_path_string();
        if let Some(light) = self.books_map.get_mut(&id) {
          light.bookmark_count = light.bookmark_count.saturating_sub(1);
        }
        needs_rebuild = true;
      }
      LibraryEvent::BookMarkUpdated { .. } => {
        needs_rebuild = true;
      }
      LibraryEvent::DirRemoved(dir) => {
        let dir_path = dir.full_path().to_string();
        // Remove all books with this parent_dir
        self.books_map.retain(|_id, light| light.parent_dir.as_ref() != dir_path);
        self.thumbnails.remove_by_parent_dir(&dir_path);
      }
    }

    if needs_rebuild {
      self.rebuild_and_sort(TargetList::Library);
      self.rebuild_and_sort(TargetList::Favorites);
      self.rebuild_and_sort(TargetList::History);
      self.rebuild_and_sort(TargetList::Bookmarks);
    }

    // Mark whether the grid should refresh its thumbnail paths. Pages observe
    // BooksState and consult `thumbnails_dirty_for_pages` to decide whether to
    // re-resolve. This is a single flag rather than per-id diffing — pages can
    // afford one `get_or_resolve` per visible id.
    let _ = thumbnails_dirty;
  }
}
