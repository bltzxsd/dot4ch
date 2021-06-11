//! # dot4ch
//!
//! dot4ch is a convenient wrapper library around 4chan's API.
//!
//! This library can fetch and update:
//! - Posts
//! - Threads
//! - Catalog
//!
//! While respecting 4chan's:  
//! - GET 1 second-per-request cooldown.
//! - `If-Modified-Since` headers with update requests.
//! - 10 second cooldown with `Thread`, `Catalog` and `Board` update requests.
//!
//! ## Example: Getting an image from the OP of a thread
//!
//! ```
//! ##[tokio::main]
//! async fn main() {
//!     use dot4ch::{Client, thread::Thread};
//!     
//!     // Making a client.
//!     let mut client = Client::new();
//!     
//!     // Building a board.
//!     let board = "g";
//!
//!     // Getting a specific `Thread` from the board.
//!     let post_id = 76759434;
//!
//!     // Fetching a new thread.
//!     let thread = Thread::new(&client, board, post_id).await.unwrap();
//!     
//!     // Getting the OP of the thread.
//!     let post = thread.op();
//!     println!("{}", post.image_url(board).unwrap());
//! }
//! ```

#![deny(
    anonymous_parameters,
    clippy::all,
    const_err,
    illegal_floating_point_literal_pattern,
    late_bound_lifetime_arguments,
    path_statements,
    patterns_in_fns_without_body,
    rust_2018_idioms,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unsafe_code,
    unused_extern_crates
)]
#![warn(
    clippy::dbg_macro,
    clippy::decimal_literal_representation,
    clippy::get_unwrap,
    clippy::missing_docs_in_private_items,
    clippy::pedantic,
    clippy::print_stdout,
    clippy::todo,
    clippy::unimplemented,
    clippy::unwrap_used,
    clippy::use_debug,
    missing_copy_implementations,
    missing_debug_implementations,
    unused_qualifications,
    variant_size_differences,
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::enum_glob_use,
    clippy::map_err_ignore,
    clippy::missing_errors_doc,
    clippy::redundant_pub_crate,
    clippy::wildcard_imports
)]
#![allow(
    clippy::missing_const_for_fn,
    clippy::must_use_candidate,
    clippy::cast_precision_loss,
    clippy::clippy::struct_excessive_bools
)]

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use log::{info, trace};
use reqwest::Response;
use std::sync::Arc;
use tokio::{
    sync::Mutex,
    time::{sleep, Duration as TkDuration},
};

pub mod board;
pub mod post;
pub mod thread;
pub mod threadlist;

/// Crate result type
pub(crate) type Result<T> = anyhow::Result<T>;

/// The main client for accessing API.
/// Handles updates, board and `reqwest::Client`
#[derive(Debug)]
pub struct Client {
    /// The creation time of the client.
    creation_time: DateTime<Utc>,
    /// The reqwest client
    req_client: reqwest::Client,
    /// The last time a client was checked
    pub last_checked: DateTime<Utc>,
}

impl Client {
    /// Make a new chan api client.
    ///
    /// This client handles your cooldown and requests internally.
    /// Thread safe.
    pub fn new() -> Arc<Mutex<Self>> {
        let req_client = reqwest::Client::new();
        let last_checked = Utc::now();
        let creation_time = last_checked;
        info!("constructed chan client.");
        Arc::new(Mutex::new(Self {
            creation_time,
            req_client,
            last_checked,
        }))
    }

    /// Returns a reference to the reqwest client in the API client.
    pub fn req_client(&self) -> &reqwest::Client {
        &self.req_client
    }

    /// Constructs and sends a GET Request to the given 4chan URL.
    ///
    /// Respects the 4chan 1 request-per-second guideline.
    ///
    /// Returns a `Response` from the given 4chan url
    ///
    /// # Errors
    ///
    ///  This function will return an error if the `GET` request to the URL fails.
    pub async fn get(&mut self, url: &str) -> Result<Response> {
        let current_time = Utc::now().signed_duration_since(self.last_checked);

        if (current_time < Duration::seconds(1)) && (self.creation_time != self.last_checked) {
            trace!("Requesting responses too fast! Slowing down requests to 1 per second");
            sleep(TkDuration::from_secs(1)).await;
        }

        let resp = self.req_client.get(url).send().await?;
        self.last_checked = Utc::now();
        trace!(
            "Updated the client last checked time: {}",
            self.last_checked
        );
        Ok(resp)
    }
}

/// Type alias for an client in an Arc<Mutex<Client>>
type Dot4chClient = Arc<Mutex<Client>>;

/// Returns an If-Modified-Since header to be used in requests.
pub(crate) async fn header(client: &Dot4chClient) -> String {
    trace!("Sending request with If-Modified-Since header");
    format!(
        "{}",
        client
            .lock()
            .await
            .last_checked
            .format("%a, %d %b %Y %T GMT")
    )
}

#[doc(hidden)]
#[async_trait(?Send)]
/// Helper trait that sends a GET request from the reqwest client
/// with a If-Modified-Since header.
pub trait IfModifiedSince {
    async fn fetch(
        client: &Dot4chClient,
        url: &str,
        header: &str,
    ) -> std::result::Result<Response, reqwest::Error>;
}

/// Update trait specifies if something can be updated or not.
///
/// By default, only Threads, Catalogs, and Boards can be updated.
///
/// # Usecase Example
/// ```
/// use dot4ch::{Client, thread::Thread, Update};
///
/// # async fn update_usecase() {
/// # let client = Client::new();
/// // Assume we have a client.
///
/// let thread = Thread::new(&client, "g", 76759434).await.unwrap();
///
/// /* --- do some work with the thread */
///
/// // time to update
/// let thread = thread.update().await.unwrap();
/// 
/// println!("{}", thread);
/// # }
/// ```
///
/// # Implementation Example
/// ```
/// # use async_trait::async_trait;
/// # use dot4ch::Update;
/// # use std::error::Error;
/// # type Result<T> = anyhow::Result<T>;
/// # type Client = std::sync::Arc<tokio::sync::Mutex<dot4ch::Client>>;
/// struct Something { pub stuff: i32 }
///
/// #[async_trait(?Send)]
/// impl Update for Something {
///     type Output = i32;
///     
///     // I will be using [`anyhow`] for error handling
///     async fn update(mut self) -> Result<Self::Output> {
///         let out = self.stuff + 32;
///         Ok(out)
///     }
/// }
/// ```
#[async_trait(?Send)]
pub trait Update {
    /// The type of the output.
    type Output;
    /// Returns the updated `self` type.
    async fn update(mut self) -> Result<Self::Output>;
}

#[async_trait(?Send)]
pub(crate) trait Procedures {
    // The Output of the impl
    type Output;

    /// Refreshes the last time the thread was accessed.
    async fn refresh_time(&mut self) -> Result<()>;

    /// Matches the `Self` 's status code to see if it has been updated,
    async fn fetch_status(mut self, response: Response) -> Result<Self::Output>;

    /// Converts a `Response` into a concrete object.
    async fn into_upper(self, response: Response) -> Result<Self::Output>;
}

#[doc(hidden)]
/// Returns the default of a type.
///
/// Used interally to generate missing fields for Post struct
fn default<T: Default>() -> T {
    Default::default()
}

/*
TODO: This function should have been implemented for (maybe something like a)
Board, or any collection
of threads, maybe turn this into trait?

/// Returns a vector of all posts that contain the given subject
pub fn find_subject(&self, subject: &str) -> Vec<&Post> {
    self.all_replies
        .iter()
        .filter(|post| post.subject().contains(subject))
        .collect::<Vec<_>>()
}

*/
