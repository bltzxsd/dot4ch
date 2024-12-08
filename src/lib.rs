#![deny(clippy::all, clippy::pedantic)]
#![deny(missing_docs)]
#![allow(clippy::must_use_candidate)]
//! # dot4ch
//!
//! dot4ch is a convenient wrapper library around an imageboard's read-only API.
//!
//! This library can fetch and update:
//! - [`Thread`]
//! - [`Catalog`]
//! - [`Board`]
//!
//! While respecting:
//! - 1 second-per-request rate-limits.
//! - `If-Modified-Since` headers with update requests.
//! - 10 seconds per thread update rate limits.
//!
//! ## Example: Printing the comment from a thread.
//!
//! ```rust
//! # type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
//! use dot4ch::thread::Post;
//! use dot4ch::thread::Thread;
//! use dot4ch::Client;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let client = Client::new();
//!     let mut thread = Thread::new(&client, "po", 570368).await?;
//!
//!     // update thread
//!     thread.update(&client).await?;
//!
//!     // print thread's OP comment
//!     let comment = thread.first().and_then(Post::com).unwrap();
//!     let comment = comment.replace("<br>", "\n"); // replace line breaks
//!     println!("op says: {comment}");
//!     Ok(())
//! }
//! ```
//!
//! [`Thread`]:  crate::models::thread::Thread
//! [`Catalog`]: crate::models::catalog::Catalog
//! [`Board`]:   crate::models::board::Board

/// Client module contains [`Client`] for requesting and updating data.
pub mod client;

/// Contains [`Error`]s that can be thrown by the libary.
///
/// [`Error`]: crate::error::Error
pub mod error;

pub(crate) mod models;

pub(crate) mod result;

pub use client::Client;
pub use models::*;
