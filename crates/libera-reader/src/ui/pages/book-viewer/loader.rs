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
}
pub const MIN_RENDER_DPI: u32 = 36;
pub const MAX_RENDER_DPI: u32 = 300;
pub const BASE_PDF_DPI: f32 = 72.0;

/// Compute optimal raster render resolution based on user zoom factor and window scale factor.
pub fn compute_target_dpi(zoom_factor: f32, scale_factor: f32) -> u32 {
  let calculated = (BASE_PDF_DPI * zoom_factor * scale_factor).round() as u32;
  calculated.clamp(MIN_RENDER_DPI, MAX_RENDER_DPI)
}

pub fn spawn_book_page_loader(
  runtime: &Runtime, cache: Arc<Mutex<BookViewerCache>>, visible_start: Arc<AtomicUsize>,
  visible_end: Arc<AtomicUsize>,
  mut load_rx: tokio::sync::mpsc::UnboundedReceiver<PageLoadRequest>,
  notify_tx: tokio::sync::mpsc::UnboundedSender<()>, coordinator: ExtractionCoordinator,
) {
  let thread_count = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4).clamp(2, 6);
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
      let coordinator_worker = coordinator.clone();

      tokio::task::spawn_blocking(move || {
        let page = req.page;
        let path = req.book_path;
        let dpi = req.dpi;

        // 1. Render page bitmap
        let img_state = match mutool::render_page_to_png_bytes(&path, page, dpi) {
          Ok(bytes) => match decode_page_image_bytes(&bytes) {
            Some(img) => PageImageState::Loaded(img),
            None => PageImageState::Failed("Failed to decode PNG bytes".to_string()),
          },
          Err(err) => PageImageState::Failed(err.to_string()),
        };

        // 2. Extract structured text layer
        let text_state = match mutool::get_page_structured_text(&path, page) {
          Ok(stext) => PageTextState::Loaded(Arc::new(stext)),
          Err(err) => PageTextState::Failed(err.to_string()),
        };

        // 3. Extract hyperlinks
        let links_state = match mutool::get_page_links(&path, page) {
          Ok(links) => PageLinksState::Loaded(Arc::new(links)),
          Err(err) => PageLinksState::Failed(err.to_string()),
        };

        {
          let mut lock = cache_worker.lock();
          lock.insert_image(page, img_state);
          lock.set_image_dpi(page, dpi);
          lock.insert_text(page, text_state);
          lock.insert_links(page, links_state);
        }

        let _ = notify_worker.send(());
        drop(permit);

        // Resume background library thumbnail extraction when no active reader pages pending
        coordinator_worker.set_mode(ExtractionCoordinatorMode::BackgroundLibrary);
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
}
