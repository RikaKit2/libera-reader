use crate::services::extraction_coordinator::{ExtractionCoordinator, ExtractionCoordinatorMode};
use crate::ui::pages::book_viewer::cache::{
  BookViewerCache, PageImageState, PageLinksState, PageTextState,
};
use crate::ui::pages::book_viewer::constants::viewport::PREFETCH_DISTANCE;
use crate::ui::pages::book_viewer::image_utils::decode_page_image_bytes;
use parking_lot::Mutex;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::runtime::Runtime;

#[derive(Debug, Clone)]
pub struct PageLoadRequest {
  pub page: usize,
  pub book_path: PathBuf,
  pub dpi: u32,
  pub book_size: u64,
  pub book_hash: Option<String>,
}
pub const MIN_RENDER_DPI: u32 = 36;
pub const MAX_RENDER_DPI: u32 = 300;
pub const BASE_PDF_DPI: f32 = 72.0;

/// Compute optimal raster render resolution based on user zoom factor and window scale factor.
pub fn compute_target_dpi(zoom_factor: f32, scale_factor: f32) -> u32 {
  let calculated = (BASE_PDF_DPI * zoom_factor * scale_factor).round() as u32;
  calculated.clamp(MIN_RENDER_DPI, MAX_RENDER_DPI)
}

/// The outcome of querying the persistent on-disk page cache.
///
/// Designed for explicit, self-documenting error handling:
/// - `Hit`: Valid cached PNG found on disk and decoded immediately (0% CPU).
/// - `Miss`: Cold cache, file does not exist yet; requires rendering.
/// - `EmptyFile`: Corrupted 0-byte file detected on disk; deleted and self-healed.
/// - `CorruptedFile`: File exists but PNG decoder failed; deleted and self-healed.
#[derive(Debug)]
pub enum DiskPageCacheResult {
  /// Valid image loaded from disk cache without executing mutool.
  Hit(Arc<gpui::RenderImage>),
  /// Cache miss: file does not exist yet on disk.
  Miss,
  /// Self-healing needed: file exists on disk but is 0 bytes.
  EmptyFile,
  /// Self-healing needed: file exists on disk but bytes could not be decoded as PNG.
  CorruptedFile,
}

impl DiskPageCacheResult {
  /// Attempts to read and decode a page image from disk in a single pass.
  pub fn probe_and_load(page_file: &std::path::Path) -> Self {
    match std::fs::read(page_file) {
      Ok(bytes) if !bytes.is_empty() => match decode_page_image_bytes(&bytes) {
        Some(img) => Self::Hit(img),
        None => Self::CorruptedFile,
      },
      Ok(_) => Self::EmptyFile,
      Err(err) if err.kind() == std::io::ErrorKind::NotFound => Self::Miss,
      Err(_) => Self::Miss,
    }
  }
}

/// Renders a page via mutool CLI, writes the resulting PNG to disk cache,
/// and decodes it into a GPU-ready `RenderImage`.
fn render_and_cache_page(
  book_path: &std::path::Path, page: usize, dpi: u32, page_file: &std::path::Path,
) -> PageImageState {
  match mutool::render_page_to_png_bytes(book_path, page, dpi) {
    Ok(bytes) => {
      if let Some(parent) = page_file.parent() {
        let _ = std::fs::create_dir_all(parent);
      }
      let _ = std::fs::write(page_file, &bytes);

      match decode_page_image_bytes(&bytes) {
        Some(img) => PageImageState::Loaded(img),
        None => PageImageState::Failed("Failed to decode rendered PNG bytes".to_string()),
      }
    }
    Err(err) => PageImageState::Failed(err.to_string()),
  }
}
#[allow(clippy::too_many_arguments)]
pub fn spawn_book_page_loader(
  runtime: &Runtime, cache: Arc<Mutex<BookViewerCache>>, visible_start: Arc<AtomicUsize>,
  visible_end: Arc<AtomicUsize>,
  mut load_rx: tokio::sync::mpsc::UnboundedReceiver<PageLoadRequest>,
  notify_tx: tokio::sync::mpsc::UnboundedSender<()>, coordinator: ExtractionCoordinator,
  app_dirs: crate::app_dirs::AppDirs, workers: usize,
) {
  // Allow user full control over worker pool, up to 100% of available CPU threads.
  let max_threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4);
  let thread_count = workers.clamp(1, max_threads);
  let semaphore = Arc::new(tokio::sync::Semaphore::new(thread_count));
  runtime.spawn(async move {
    while let Some(req) = load_rx.recv().await {
      // Prioritize active reader tasks over background library extraction
      coordinator.set_mode(ExtractionCoordinatorMode::ForegroundReader);

      let permit = match semaphore.clone().acquire_owned().await {
        Ok(p) => p,
        Err(_) => break,
      };

      let s = visible_start.load(Ordering::Relaxed);
      let e = visible_end.load(Ordering::Relaxed);

      // Only cancel requests if we have an established visible range and page is far outside
      let is_out_of_range = e != 0
        && (req.page < s.saturating_sub(PREFETCH_DISTANCE) || req.page > (e + PREFETCH_DISTANCE));

      if is_out_of_range {
        let mut lock = cache.lock();
        lock.pop_image_loading(req.page);
        lock.pop_text_loading(req.page);
        drop(permit);
        continue;
      }

      let cache_worker = cache.clone();
      let notify_worker = notify_tx.clone();
      let _coordinator_worker = coordinator.clone();
      let app_dirs_worker = app_dirs.clone();
      tokio::task::spawn_blocking(move || {
        let page = req.page;
        let path = req.book_path;
        let dpi = req.dpi;
        let book_size = req.book_size;
        let book_hash = req.book_hash;

        let page_file = app_dirs_worker.book_page_path(book_size, book_hash.as_deref(), page, dpi);

        // 1. Check disk cache with explicit, self-documenting outcome handling
        let img_state = match DiskPageCacheResult::probe_and_load(&page_file) {
          DiskPageCacheResult::Hit(img) => {
            // Fast path: loaded directly from disk cache in 1 syscall (0% CPU)
            PageImageState::Loaded(img)
          }
          DiskPageCacheResult::Miss => {
            // Cold cache: first time rendering this page
            render_and_cache_page(&path, page, dpi, &page_file)
          }
          DiskPageCacheResult::EmptyFile => {
            // Self-healing: 0-byte artifact from interrupted write; purge and regenerate
            tracing::warn!(page, path = %page_file.display(), "Purging empty 0-byte cached page to self-heal");
            let _ = std::fs::remove_file(&page_file);
            render_and_cache_page(&path, page, dpi, &page_file)
          }
          DiskPageCacheResult::CorruptedFile => {
            // Self-healing: invalid/corrupted PNG data; purge and regenerate
            tracing::warn!(page, path = %page_file.display(), "Purging corrupted cached page to self-heal");
            let _ = std::fs::remove_file(&page_file);
            render_and_cache_page(&path, page, dpi, &page_file)
          }
        };

        // 2. Extract structured text layer only once per page (DPI independent)
        let need_text = {
          let mut lock = cache_worker.lock();
          !matches!(lock.get_text(page), Some(PageTextState::Loaded(_)))
        };
        let text_state = if need_text {
          match mutool::get_page_structured_text(&path, page) {
            Ok(stext) => Some(PageTextState::Loaded(Arc::new(stext))),
            Err(err) => Some(PageTextState::Failed(err.to_string())),
          }
        } else {
          None
        };

        // 3. Extract hyperlinks only once per page (DPI independent)
        let need_links = {
          let mut lock = cache_worker.lock();
          !matches!(lock.get_links(page), Some(PageLinksState::Loaded(_)))
        };
        let links_state = if need_links {
          match mutool::get_page_links(&path, page) {
            Ok(links) => Some(PageLinksState::Loaded(Arc::new(links))),
            Err(err) => Some(PageLinksState::Failed(err.to_string())),
          }
        } else {
          None
        };
        {
          let mut lock = cache_worker.lock();
          lock.insert_image(page, img_state);
          lock.set_image_dpi(page, dpi);
          if let Some(ts) = text_state {
            lock.insert_text(page, ts);
          }
          if let Some(ls) = links_state {
            lock.insert_links(page, ls);
          }
        }

        let _ = notify_worker.send(());
        drop(permit);
      });
    }
  });
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_compute_target_dpi() {
    assert_eq!(compute_target_dpi(1.0, 1.0), 72);
    assert_eq!(compute_target_dpi(1.5, 1.0), 108);
    assert_eq!(compute_target_dpi(2.0, 1.0), 144);
    assert_eq!(compute_target_dpi(1.0, 2.0), 144);
    assert_eq!(compute_target_dpi(1.5, 2.0), 216);
    assert_eq!(compute_target_dpi(0.1, 1.0), 36);
    assert_eq!(compute_target_dpi(5.0, 2.0), 300);
  }

  #[test]
  fn test_disk_page_cache_result() {
    let tmp = tempfile::tempdir().unwrap();
    let missing = tmp.path().join("missing.png");
    assert!(matches!(DiskPageCacheResult::probe_and_load(&missing), DiskPageCacheResult::Miss));

    let empty = tmp.path().join("empty.png");
    std::fs::write(&empty, []).unwrap();
    assert!(matches!(DiskPageCacheResult::probe_and_load(&empty), DiskPageCacheResult::EmptyFile));

    let corrupt = tmp.path().join("corrupt.png");
    std::fs::write(&corrupt, b"not a valid png file").unwrap();
    assert!(matches!(
      DiskPageCacheResult::probe_and_load(&corrupt),
      DiskPageCacheResult::CorruptedFile
    ));
  }
}
