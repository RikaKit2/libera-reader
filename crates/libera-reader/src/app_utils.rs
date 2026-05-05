use gpui::{AsyncApp, Hsla};

use crate::TOKIO;
use gpui::App;
use libera_reader_core::ctx::Ctx;

pub(crate) fn set_lang(cx: &mut App) {
  let ctx = Ctx::global(cx);
  let lang = ctx.settings.read().language.to_string();
  rust_i18n::set_locale(lang);
  cx.refresh_windows();
}

pub fn adjust_brightness(color: Hsla, factor: f32) -> Hsla {
  Hsla { h: color.h, s: color.s, l: (color.l * factor).clamp(0.0, 1.0), a: color.a }
}

pub(crate) fn start_services(cx: &mut App) {
  let scan_service = Ctx::global(cx).services.scan_service.clone();

  cx.spawn(|async_app: &mut AsyncApp| {
    let owned_app = async_app.clone();

    async move {
      let tokio_rt = TOKIO.get().unwrap();

      tokio_rt.spawn(async move {
        if let Err(e) = scan_service.run().await {
          eprintln!("ScanService error: {:?}", e);
        }
      });

      let _ = owned_app.update(|cx| {
        let _guard = tokio_rt.enter();

        let ctx = Ctx::global_mut(cx);

        if let Err(e) = ctx.db.compact() {
          eprintln!("DB Compact error: {:?}", e);
        }

        if let Err(e) = ctx.services.notify_service.run() {
          eprintln!("NotifyService error: {:?}", e);
        }

        // Start thumbnail extraction
        ctx.services.run_data_extraction_service();
      });
    }
  })
  .detach();
}
