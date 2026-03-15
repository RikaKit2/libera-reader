use gpui::App;
use libera_reader_core::ctx::Ctx;

pub(crate) fn set_lang(cx: &mut App) {
  let ctx = Ctx::global(cx);
  let lang = ctx.settings.read().language.to_string();
  rust_i18n::set_locale(lang);
  cx.refresh_windows();
}
