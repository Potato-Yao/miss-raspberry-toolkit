#![warn(clippy::all, rust_2018_idioms)]

rust_i18n::i18n!("locales", fallback = "en");

mod app;
pub mod sensor_data;
pub mod views;

pub use app::App;
