//! API types shared across backend, frontend, and WASM modules.
//!
//! This crate provides all request, response, and validation types used in the API.
//! Types are designed to work with serde for JSON serialization and optionally
//! with utoipa for OpenAPI schema generation.

mod enums;
mod requests;
mod responses;
mod validation;

pub use enums::*;
pub use requests::*;
pub use responses::*;
pub use validation::*;
