//! Application services: orchestrate repositories and apply domain rules.
//! Handlers call into these and only translate HTTP ⇄ DTOs.

pub mod academics;
pub mod auth;
pub mod dashboard;
pub mod dictionary;
pub mod engagement;
pub mod lessons;
pub mod records;
pub mod snapshot;
pub mod stats;
