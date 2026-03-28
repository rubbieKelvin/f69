//! Feature-flag microservice: projects, flags, and JWT-gated routes.
//!
//! The binary in `main.rs` composes Axum with this library.

pub mod auth_user;
pub mod entity;
pub mod error;
pub mod jwks_cache;
pub mod routes;
pub mod state;
