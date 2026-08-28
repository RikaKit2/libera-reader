use gpui::{AsyncApp, Context, WeakEntity};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

/// A thread-safe, reusable debouncer utility for GPUI async tasks.
///
/// Tracks generation counters so that out-of-order or superseded debounced
/// tasks are cleanly discarded before (and inside) executing GPUI state updates.
#[derive(Debug, Clone, Default)]
pub struct Debouncer {
  generation: Arc<AtomicU64>,
}

impl Debouncer {
  /// Create a new `Debouncer`.
  pub fn new() -> Self {
    Self { generation: Arc::new(AtomicU64::new(0)) }
  }

  /// Advance to the next generation and return the new generation ID.
  pub fn next_generation(&self) -> u64 {
    self.generation.fetch_add(1, Ordering::SeqCst) + 1
  }

  /// Get the current generation ID.
  pub fn current_generation(&self) -> u64 {
    self.generation.load(Ordering::SeqCst)
  }

  /// Check if the given generation is the current latest generation.
  pub fn is_current(&self, generation_id: u64) -> bool {
    self.generation.load(Ordering::SeqCst) == generation_id
  }

  /// Reset the generation counter to 0.
  pub fn reset(&self) {
    self.generation.store(0, Ordering::SeqCst);
  }

  /// Spawn a debounced action on a GPUI entity context.
  ///
  /// Advances the generation counter. After `delay`, if no newer generation
  /// has been requested, `action` will be executed on the entity.
  pub fn debounce<T: 'static>(
    &self, cx: &mut Context<T>, delay: Duration,
    action: impl FnOnce(&mut T, &mut Context<T>) + Send + 'static,
  ) {
    let target_gen = self.next_generation();
    let generation = self.generation.clone();

    cx.spawn(move |this: WeakEntity<T>, cx: &mut AsyncApp| {
      let mut owned_cx = cx.clone();
      async move {
        owned_cx.background_executor().timer(delay).await;
        if generation.load(Ordering::SeqCst) != target_gen {
          return;
        }

        let _ = this.update(&mut owned_cx, |entity, cx| {
          if generation.load(Ordering::SeqCst) == target_gen {
            action(entity, cx);
          }
        });
      }
    })
    .detach();
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_debouncer_generations() {
    let debouncer = Debouncer::new();
    assert_eq!(debouncer.current_generation(), 0);
    assert!(debouncer.is_current(0));

    let gen1 = debouncer.next_generation();
    assert_eq!(gen1, 1);
    assert_eq!(debouncer.current_generation(), 1);
    assert!(debouncer.is_current(1));
    assert!(!debouncer.is_current(0));

    let gen2 = debouncer.next_generation();
    assert_eq!(gen2, 2);
    assert!(debouncer.is_current(2));
    assert!(!debouncer.is_current(gen1));

    debouncer.reset();
    assert_eq!(debouncer.current_generation(), 0);
    assert!(debouncer.is_current(0));
    assert!(!debouncer.is_current(gen2));
  }

  #[test]
  fn test_debouncer_clone_shares_state() {
    let debouncer1 = Debouncer::new();
    let debouncer2 = debouncer1.clone();
    let gen_id = debouncer1.next_generation();
    assert_eq!(debouncer2.current_generation(), gen_id);
    assert!(debouncer2.is_current(gen_id));
  }
}
