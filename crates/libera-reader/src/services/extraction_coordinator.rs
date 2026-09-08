use std::sync::Arc;
use tokio::sync::watch::{Receiver, Sender, channel};

/// Operational mode of the background extraction coordinator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExtractionCoordinatorMode {
  /// User is browsing library: background cover thumbnail extraction is active
  #[default]
  BackgroundLibrary,
  /// User is actively viewing a book: resources are prioritized for book pages and structured text
  ForegroundReader,
  /// Background extraction services are paused
  Paused,
}

/// Coordinates CPU and disk I/O between background cover extraction and interactive book reading.
#[derive(Clone)]
pub struct ExtractionCoordinator {
  tx: Arc<Sender<ExtractionCoordinatorMode>>,
  rx: Receiver<ExtractionCoordinatorMode>,
}

impl gpui::Global for ExtractionCoordinator {}

impl Default for ExtractionCoordinator {
  fn default() -> Self {
    Self::new()
  }
}

impl ExtractionCoordinator {
  /// Initialize coordinator in default BackgroundLibrary mode.
  pub fn new() -> Self {
    let (tx, rx) = channel(ExtractionCoordinatorMode::BackgroundLibrary);
    Self { tx: Arc::new(tx), rx }
  }

  /// Query the current coordinator mode.
  pub fn mode(&self) -> ExtractionCoordinatorMode {
    *self.rx.borrow()
  }

  /// Transition coordinator to a new operational mode.
  pub fn set_mode(&self, new_mode: ExtractionCoordinatorMode) {
    match self.mode() == new_mode {
      true => {}
      false => {
        let _ = self.tx.send(new_mode);
      }
    }
  }

  /// Create a new watch receiver subscription.
  pub fn subscribe(&self) -> Receiver<ExtractionCoordinatorMode> {
    self.rx.clone()
  }

  /// Asynchronously wait until the coordinator enters BackgroundLibrary mode.
  pub async fn wait_for_library_mode(rx: &mut Receiver<ExtractionCoordinatorMode>) {
    let current_mode = *rx.borrow_and_update();
    match current_mode {
      ExtractionCoordinatorMode::BackgroundLibrary => {}
      ExtractionCoordinatorMode::ForegroundReader | ExtractionCoordinatorMode::Paused => {
        let _ =
          rx.wait_for(|mode| matches!(mode, ExtractionCoordinatorMode::BackgroundLibrary)).await;
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn test_coordinator_transitions() {
    let coordinator = ExtractionCoordinator::new();
    assert_eq!(coordinator.mode(), ExtractionCoordinatorMode::BackgroundLibrary);

    coordinator.set_mode(ExtractionCoordinatorMode::ForegroundReader);
    assert_eq!(coordinator.mode(), ExtractionCoordinatorMode::ForegroundReader);

    let mut sub = coordinator.subscribe();
    coordinator.set_mode(ExtractionCoordinatorMode::BackgroundLibrary);
    ExtractionCoordinator::wait_for_library_mode(&mut sub).await;
    assert_eq!(*sub.borrow(), ExtractionCoordinatorMode::BackgroundLibrary);
  }
}
