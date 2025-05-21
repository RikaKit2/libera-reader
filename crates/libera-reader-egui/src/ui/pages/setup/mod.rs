use crate::router::{RootRoute, Route, Router};
use egui::Context;
use libera_reader_core::db::models::Settings;
use libera_reader_core::types::DB;
use rfd::FileDialog;

pub(crate) struct Setup {}
impl Setup {
  pub(crate) fn new() -> Self {
    Self {}
  }
  pub(crate) fn make(&self, ctx: &Context, settings: &mut Settings, db: &DB, router: &mut Router) {
    egui::CentralPanel::default().show(&ctx, |ui| {
      if ui.button("Select directory to scan").clicked() {
        if let Some(path) = FileDialog::new().pick_folder() {
          settings.set_path_to_scan(path.display().to_string(), db);
        }
      }
      match &settings.path_to_scan {
        None => {}
        Some(dir) => { ui.label(format!("Directory to scan: {}", dir)); }
      }
      if ui.button("Next").clicked() {
        if settings.path_to_scan.is_some() {
          router.set_route(RootRoute::Base(Route::Library));
        }
      }
    });
  }
}
