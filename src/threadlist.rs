//! A list of all avaliable threads in a Board
//!
//! This is documented as `threads.json` in the
//! [4chan API Repository](<https://github.com/4chan/4chan-API/blob/master/pages/Threadlist.md>)
//!
//! However it contains pretty much the same level of information from the
//! [4chan-API/Catalog](<https://github.com/4chan/4chan-API/blob/master/pages/Catalog.md>)
//!
//! Except recent replies, which can already be accessed by [`Thread`] and [`Post`]'s
//! functionality.
//! The `threads.json` file is a comprehensive list of all threads that contains:
//! - The thread OP number
//! - The index page the thread is currently on
//! - A UNIX timestamp marking the last time the thread was modified
//! - The number of replies a thread has
//!

use crate::{header, Dot4chClient, IfModifiedSince, Procedures, Update};
use async_trait::async_trait;
use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use log::debug;
use reqwest::{header::IF_MODIFIED_SINCE, Response, StatusCode};
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};
use tokio::time;

/// A summarized list of all threads on a board including
/// thread numbers, their modification time and reply count.
///
/// # Example
///
/// ```
/// # async fn catalog_check() {
/// # use dot4ch::{threadlist::Catalog, Client};
/// # let client = Client::new();
/// let catalog = Catalog::new(&client, "g").await.unwrap();
///
/// // print the first page
///
/// println!("{:?}", catalog.page(0));
/// # }
/// ```
///
/// to get all threads from catalog
#[derive(Debug)]
pub struct Catalog {
    /// The board of the catalog
    board: String,
    /// The pages of the catalog which contain threads
    threads: Vec<Page>,
    /// The time when catalog was accessed
    last_accessed: DateTime<Utc>,
    /// client
    client: Dot4chClient,
}

#[derive(Debug, Deserialize, Serialize, Default)]
/// A Page in the catalog.
/// Pages contain their own number and a vector or `CatalogThreads`
///
/// This is usually the intermediate between a Catalog and a `CatalogThread`
pub struct Page {
    /// The page number that the following thread array is on
    page: u8,
    /// The array of thread objects
    threads: Vec<CatalogThread>,
}

impl Page {
    /// Returns the threads in the catalog.
    pub fn threads(self) -> Vec<CatalogThread> {
        self.threads
    }

    /// Returns the page number of a page.
    pub fn num(self) -> u8 {
        self.page
    }
}

impl Display for Catalog {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let fmt = format!(
            "Board: /{}/\nLast accessed: {}\nPages: {}",
            self.board,
            self.last_accessed.format("%a, %d %b %Y %T").to_string(),
            self.threads.iter().map(Page::to_string).collect::<String>()
        );
        write!(f, "{}", fmt)
    }
}

impl Display for Page {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let fmt = format!(
            "\nPage Number: {}\nThreads: {}",
            self.page,
            self.threads
                .iter()
                .map(CatalogThread::to_string)
                .collect::<String>()
        );
        write!(f, "{}", fmt)
    }
}

#[async_trait(?Send)]
impl Update for Catalog {
    type Output = Self;
    /// Returns an updated catalog.
    async fn update(mut self) -> crate::Result<Self> {
        self.refresh_time().await?;

        let updated_catalog = {
            let header = header(&self.client).await;
            let get_url = format!("https://a.4cdn.org/{}/threads.json", &self.board);
            let response = Self::fetch(&self.client, &get_url, &header).await?;

            self.client.lock().await.last_checked = Utc::now();

            self.fetch_status(response).await?
        };

        Ok(updated_catalog)
    }
}

#[async_trait(?Send)]
impl Procedures for Catalog {
    type Output = Self;
    /// Refreshes the time in the `Thread` instance.
    /// also handles the sleep of the thread update.
    ///
    /// # Errors
    ///
    /// This function should probably not fail **but** can fail
    /// if
    async fn refresh_time(&mut self) -> crate::Result<()> {
        let curr = Utc::now().signed_duration_since(self.last_accessed);
        if curr < Duration::seconds(10) {
            debug!(
                "Updating Catalog too quickly! Waiting for {} seconds",
                (10000_f32 - curr.num_milliseconds() as f32) / 1000_f32
            );
            match Duration::seconds(10).checked_sub(&curr) {
                Some(time) => time::sleep(time.to_std()?).await,
                None => return Err(anyhow::anyhow!("Overflow in subtraction of `Duration`")),
            }
        }
        Ok(())
    }

    /// Updates the status of a `Response` and generates a new Catalog if needed.
    async fn fetch_status(mut self, response: Response) -> crate::Result<Self::Output> {
        Ok(match response.status() {
            StatusCode::OK => self.into_upper(response).await?,
            StatusCode::NOT_MODIFIED => {
                self.last_accessed = Utc::now();
                self
            }
            other => {
                return Err(anyhow::anyhow!(
                    "Unexpected StatusCode on Catalog Update: {}",
                    other
                ))
            }
        })
    }

    /// Converts the `Response` into a `Catalog`
    async fn into_upper(self, response: Response) -> crate::Result<Self::Output> {
        let threads = response.json::<Vec<Page>>().await?;
        let last_accessed = Utc::now();
        Ok(Self {
            threads,
            last_accessed,
            board: self.board.to_string(),
            client: self.client.clone(),
        })
    }
}

#[async_trait(?Send)]
impl IfModifiedSince for Catalog {
    async fn fetch(
        client: &Dot4chClient,
        url: &str,
        header: &str,
    ) -> Result<Response, reqwest::Error> {
        let response = client
            .lock()
            .await
            .req_client()
            .get(url)
            .header(IF_MODIFIED_SINCE, header)
            .send()
            .await;
        response
    }
}

impl Catalog {
    /// Returns a new `ThreadList` from a given board.
    ///
    /// # Errors
    ///
    /// This function will return an error if the board isn't valid
    pub async fn new(client: &Dot4chClient, board: &str) -> crate::Result<Self> {
        let url = format!("https://a.4cdn.org/{}/threads.json", board);
        let threads = client.lock().await.get(&url).await?;

        threads
            .error_for_status_ref()
            .map_err(anyhow::Error::from)?;

        let threads = threads.json::<Vec<Page>>().await?;

        Ok(Self {
            threads,
            last_accessed: Utc::now(),
            board: board.to_string(),
            client: client.clone(),
        })
    }

    /// Updates the last accessed time to be the current time.
    pub fn update_time(mut self) {
        self.last_accessed = Utc::now();
    }

    /// Returns a reference to the Page if it exists. None otherwise
    pub fn page(&self, index: usize) -> Option<&Page> {
        self.threads.get(index)
    }

    /// Returns all the pages from the catalog.
    pub fn all_pages(self) -> Vec<Page> {
        self.threads
    }
}

/// Contains some metadata about the thread.
///
/// Usually used in the context of a [`Page`]
#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Clone, Copy)]
pub struct CatalogThread {
    /// The OP ID of a thread
    no: u32,
    /// The UNIX timestamp marking the last time the thread was modified
    /// (post added/modified/deleted, thread closed/sticky settings modified)
    last_modified: i64,
    /// A numeric count of the number of replies in the thread
    replies: u32,
}

impl CatalogThread {
    /// Returns the thread number.
    pub fn id(&self) -> u32 {
        self.no
    }

    /// Returns the UNIX timestamp of when the thread was last modified.
    pub fn last_modified(&self) -> i64 {
        self.last_modified
    }

    /// Returns the number of replies in a thread.
    pub fn replies(&self) -> u32 {
        self.replies
    }

    /// Convert a [`CatalogThread`] into a [`Thread`]
    ///
    /// # Errors
    ///
    /// This function will fail if the request to fetch the [`Thread`] fails.
    pub async fn to_thread(self, client: &Dot4chClient, board: &str) -> anyhow::Result<Thread> {
        Ok(Thread::new(client, board, self.no).await?)
    }
}

impl Display for CatalogThread {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let g = NaiveDateTime::from_timestamp(self.last_modified, 0);
        let fmt = format!(
            "\n\tThread ID: {} | Last Modified: {} | Number of Replies: {}",
            self.no, g, self.replies
        );
        write!(f, "{}", fmt)
    }
}

/* depending on argument.
///
/// Uses the `get()` method on `Vec`.
///
/// - Returns `None` if the provided index is out of bounds.
/// - Returns a single element if a single index is provided.
/// - Returns a slice of elements if a range is provided.
///
/// # Example
///
/// ```ignore
/// let catalog = Catalog::new(&client, "g").await?;
/// println!("{:?}", thread.get(1..4));
/// ```
 */
