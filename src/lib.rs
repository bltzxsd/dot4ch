//! # dot4ch
//!
//!  dot4ch is a convenient wrapper library around 4chan's API.
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
//! ```rust
//! #[tokio::main]
//! async fn main() {
//!     let mut client = Client::new();
//!
//!     let board = "g";
//!
//!     let post_id = 81743559;
//!
//!     let thread = Thread::new(&mut client, board, post_id).unwrap();
//!     
//!     let post = thread.op();
//!     println!("{}", post.image_url().unwrap());
//! }
//! ```

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use log::{debug, info, trace};
use reqwest::Response;
use std::error::Error;
use std::result;
use std::sync::Arc;
use tokio::{
    sync::Mutex,
    time::{sleep, Duration as TkDuration},
};

pub mod board;
pub mod post;
pub mod thread;
pub mod threadlist;

pub(crate) type Result<T> = result::Result<T, Box<dyn Error>>;

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
    pub async fn get(&mut self, url: &str) -> Result<Response> {
        let current_time = Utc::now().signed_duration_since(self.last_checked);

        if (current_time < Duration::seconds(1)) && (self.creation_time != self.last_checked) {
            info!("Requesting responses too fast! Slowing down requests to 1 per second");
            sleep(TkDuration::from_secs(1)).await;
        }

        let resp = self.req_client.get(url).send().await?;
        self.last_checked = Utc::now();
        debug!(
            "Updated the client last checked time: {}",
            self.last_checked
        );
        Ok(resp)
    }
}


/// Returns an If-Modified-Since header to be used in requests.
pub async fn header(client: &mut Arc<tokio::sync::Mutex<Client>>) -> String {
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
pub trait IfModifiedSince {
    async fn fetch(
        client: &&mut Arc<tokio::sync::Mutex<Client>>,
        url: &str,
        header: &str,
    ) -> std::result::Result<Response, reqwest::Error>;
}


/// Update trait specifies if something can be updated or not. 
/// 
/// By default, only Threads, Catalogs, and Boards can be updated.
/// 
/// # Example 
/// ```
/// use async_trait::async_trait; 
/// type Client = std::sync:Arc<tokio::sync::Mutex<crate::Client>>;
/// struct Something { stuff: i32 }
/// 
/// #[async_trait(?Send)]
/// impl Update for Something {
///     type Output = i32;
///     async fn update(mut self, client: &mut Client) -> Result<Self::Output>; {
///         let out = self.stuff + 32;
///         Ok(out)
///     }
/// }
/// ```
#[async_trait(?Send)]
pub trait Update {
    /// The type of the output
    type Output;
    /// Returns the updated `self` type.
    async fn update(mut self, client: &mut Arc<tokio::sync::Mutex<Client>>)
        -> Result<Self::Output>;
}

#[doc(hidden)]
fn default<T: Default>() -> T {
    Default::default()
}
