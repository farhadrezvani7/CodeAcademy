//! Data access. Every query is parameterized (`$n` binds); no SQL is built
//! from user input. Functions accept any executor so services can compose
//! them inside a transaction.

pub mod accounts;
pub mod activity;
pub mod catalog;
pub mod content;
pub mod student;
