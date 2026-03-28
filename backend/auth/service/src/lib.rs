//! Auth microservice library: HTTP routes, SeaORM entities, and shared `AppState`.
//!
//! The binary in `main.rs` wires Axum, middleware, and this crate.

pub mod entity;
pub mod error;
pub mod routes;
pub mod state;
