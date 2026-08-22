use std::sync::OnceLock;
use tokio::runtime::Runtime;

pub static TOKIO: OnceLock<Runtime> = OnceLock::new();

rust_i18n::i18n!("../../locales");

pub mod app_dirs;
pub mod app_utils;
pub mod books_state;
pub mod ctx;
pub mod db;
pub mod error_handler;
pub mod not_cached_books;
pub mod services;
pub mod settings;
pub mod theme;
pub mod types;
pub mod ui;
pub mod utils;
