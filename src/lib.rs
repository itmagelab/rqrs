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
//! ## Example
//!
//! The example below demonstrates how to send a GET request to the [reqres.in](https://reqres.in) API,
//! attach headers, set query parameters, and pretty-print the JSON response:
//!
//! ```rust
//! use rqrs::prelude::*;
//! use serde_json;
//! use crate::Result;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let rq = Rq::from_static("https://reqres.in")?
//!         .uri("/api/users")
//!         .method("GET")?
//!         .add_secret_header(b"x-api-key", "reqres-free-v1")?
//!         .add_header(b"Content-Type", "application/json")?
//!         .add_params(vec![("page", "2")]);
//!
//!     let rs = rq.apply().await?;
//!     let json = serde_json::to_string_pretty(&rs.data)?;
//!     println!("{json}");
//!
//!     Ok(())
//! }
//! ```
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

use std::{env, sync::Arc};

pub mod ai;
pub mod api;
pub mod prelude;

pub type Result<T> = anyhow::Result<T>;

#[derive(Debug)]
pub struct Bot {
    pub url: Arc<url::Url>,
    pub debug: bool,
}

/// Create a new bot from environment variables
pub fn from_env_handler() -> Result<Bot> {
    dotenv::dotenv().ok();

    let url = env::var("RQRS_URL").unwrap_or("http://localhost:3000".into());
    let debug = env::var("RQRS_DEBUG")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(false);
    let mut bot = Bot::new(url)?;
    bot.debug = debug;
    Ok(bot)
}

impl Bot {
    /// Create a new bot
    pub fn new<S>(url: S) -> Result<Self>
    where
        S: AsRef<str>,
    {
        Ok(Self {
            url: Arc::from(url::Url::parse(url.as_ref())?),
            debug: false,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bot_builder() {
        dotenv::dotenv().ok();

        let bot = Bot::new("http://localhost:3000").unwrap();
        assert_eq!(bot.url.as_str(), "http://localhost:3000/");
        assert_eq!(bot.url.port(), Some(3000));

        let bot = crate::from_env_handler().unwrap();
        dbg!(bot.debug);
        assert!(!bot.debug);
    }
}
