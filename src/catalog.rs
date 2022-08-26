//! A list of all avaliable threads in a Board
//!
//! This is documented as `threads.json` in the
//! [4chan API Repository](<https://github.com/4chan/4chan-API/blob/master/pages/Threadlist.md>)
//!
//! However it contains pretty much the same level of information from the
//! [4chan-API/Catalog](<https://github.com/4chan/4chan-API/blob/master/pages/Catalog.md>)
//!
//! Except recent replies, which can already be accessed by [`Thread`] and [`crate::post::Post`]'s
//! functionality.
//! The `threads.json` file is a comprehensive list of all threads that contains:
//! - The thread OP number
//! - The index page the thread is currently on
//! - A UNIX timestamp marking the last time the thread was modified
//! - The number of replies a thread has
//!

use crate::{
    cat_thread::Page, error::DotError, header, Dot4chClient, IfModifiedSince, Procedures, Update, Result
};
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use log::debug;
use reqwest::{header::IF_MODIFIED_SINCE, Response, StatusCode};
use tokio::time;

/// A summarized list of all threads on a board including
/// thread numbers, their modification time and reply count.
///
/// # Example
///
/// ```
/// # async fn catalog_check() {
/// # use dot4ch::{Catalog, Client};
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

impl Catalog {
    /// Returns a new [`Catalog`] from a given board.
    ///
    /// # Errors
    ///
    /// This function will return an error if the board isn't valid
    pub async fn new(client: &Dot4chClient, board: &str) -> Result<Self> {
        let url = format!("https://a.4cdn.org/{}/threads.json", board);
        let threads = client.lock().await.get(&url).await?;

        threads.error_for_status_ref().map_err(DotError::Reqwest)?;

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

    /// Returns a reference to the [`Page`] if it exists. None otherwise
    pub fn page(&self, index: usize) -> Option<&Page> {
        self.threads.get(index)
    }

    /// Returns all the [`Page`]s from the [`Catalog`].
    pub fn all_pages(self) -> Vec<Page> {
        self.threads
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
    /// if the subtraction from [`time::Duration`] results in an overflow
    /// which normally should not occur, in which case, the program will panic
    async fn refresh_time(&mut self) -> crate::Result<()> {
        let since = Utc::now().signed_duration_since(self.last_accessed);
        if since < Duration::seconds(10) {
            let remaining = 10 - since.num_seconds();
            debug!(
                "Updating Catalog too quickly! Waiting for {} seconds",
                remaining
            );
            if let Some(time) = Duration::seconds(10).checked_sub(&since) {
                time::sleep(
                    time.to_std()
                        .expect("could not convert chrono time to stdtime"),
                )
                .await;
            }
        }
        Ok(())
    }

    /// Updates the status of a [`Response`] and generates a new [`Catalog`] if needed.
    async fn fetch_status(self, response: Response) -> Result<Self::Output> {
        match response.status() {
            StatusCode::OK => Ok(self.from_response(response).await?),
            StatusCode::NOT_MODIFIED => {
                let mut catalog = self;
                catalog.last_accessed = Utc::now();
                Ok(catalog)
            }
            other => Err(DotError::Update(format!("recieved invalid status code: {}", other))),
        }
    }

    /// Converts the [`Response`] into a [`Catalog`]
    async fn from_response(self, response: Response) -> crate::Result<Self::Output> {
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
    ) -> Result<Response> {
        client
            .lock()
            .await
            .req_client()
            .get(url)
            .header(IF_MODIFIED_SINCE, header)
            .send()
            .await
            .map_err(DotError::Reqwest)
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

impl<Idx> std::ops::Index<Idx> for Catalog
where Idx: std::slice::SliceIndex<[Page]> {
    type Output = Idx::Output;
    fn index(&self, index: Idx) -> &Self::Output {
        &self.threads[index]
    }
}


#[cfg(feature = "display")]
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
