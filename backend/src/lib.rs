pub mod api;
pub mod auth;
pub mod config;
pub mod domain;
pub mod error;
pub mod repo;
pub mod seed;
pub mod services;
pub mod state;

pub use api::router;
pub use state::{AppState, Clock};

pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");
