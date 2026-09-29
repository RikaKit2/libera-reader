#![recursion_limit = "1024"]

use std::sync::OnceLock;
use tokio::runtime::Runtime;

pub static TOKIO: OnceLock<Runtime> = OnceLock::new();
pub use gpui_kit;

rust_i18n::i18n!("../../locales");

pub mod app_dirs;
pub mod app_ext;
pub mod books_state;
pub mod db;
pub mod not_cached_books;
pub mod services;
pub mod settings;
pub mod types;
pub mod ui;
pub mod utils;
