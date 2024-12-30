use crate::types::{NotifyEvents, NotifyWatcher};
use notify::{RecursiveMode, Watcher};
use std::sync::{Arc, Mutex};
use crate::vars::WATCHDOG;

pub(crate) struct Watchdog {
  pub(crate) events: NotifyEvents,
  pub(crate) inn: NotifyWatcher,
}

impl Watchdog {
  pub(crate) fn new() -> Self {
    let (tx, events) = crossbeam_channel::unbounded();
    let watcher = Arc::from(Mutex::from(
      notify::recommended_watcher(move |res| { tx.send(res).unwrap(); }).unwrap()
    ));
    Self { events, inn: watcher }
  }
}

impl Default for Watchdog {
  fn default() -> Self {
    Self::new()
  }
}

pub fn run(path_to_scan: &String) {
  WATCHDOG.inn.lock().unwrap().watch(path_to_scan.as_ref(), RecursiveMode::Recursive).unwrap();
}
pub fn stop(path_to_scan: &String) -> Result<(), notify::Error> {
  match WATCHDOG.inn.lock()?.unwatch(path_to_scan.as_ref()) {
    Ok(_) => {
      Ok(())
    }
    Err(e) => {
      Err(e)
    }
  }
}
