//! # RQRS
//!
//! RQRS is a lightweight HTTP client utility designed to simplify building and executing HTTP requests.
//! It provides a fluent, chainable API to configure requests, including headers, query parameters, and body.
//! The library is built on top of `reqwest`, leveraging its async HTTP capabilities while offering a simpler, more ergonomic interface.
//!
//! It provides a fluent, chainable API to configure:
//! - Base URLs and endpoints
//! - HTTP methods (GET, POST, etc.)
//! - Query parameters
//! - Custom and secret headers
//!
//! This makes it useful for prototyping, interacting with REST APIs, or building internal API clients.
//!
//! ## Notes
//! - `add_secret_header` is intended for sensitive headers (e.g., API keys), and may be masked in logs if logging is enabled.
//! - The response body (`rs.data`) must implement `Serialize` to use `serde_json::to_string_pretty`.
//!
//! ## Future Extensions
//! This utility may be extended to support:
//! - Request bodies (e.g., JSON, form data)
//! - Custom timeouts
//! - Retry logic
//! - Middleware or interceptors

pub mod ai;
pub mod api;
pub mod prelude;

pub type Result<T> = anyhow::Result<T>;
