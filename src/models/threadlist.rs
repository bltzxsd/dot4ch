use crate::{
    client::Reply,
    error::Error::{self, MissingHeader},
    models::Metadata,
    result::Result,
    Client,
};
use reqwest::header::LAST_MODIFIED;
use serde::{Deserialize, Serialize};

/// Represents the top-level structure of threadslist.
///
/// Contains a vector of [`BaseThread`]s.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadList {
    threads: Vec<BaseThread>,
    #[serde(skip)]
    pub(crate) metadata: Metadata,
}

impl ThreadList {
    /// Constructs a `ThreadList` by fetching data from `threadlist.json`
    ///
    /// # Errors
    ///
    /// This function will return an error if the client fails to fetch the data
    /// or if necessary headers/content is missing from the response.
    pub async fn new(client: &Client, board: &str) -> Result<Self> {
        let url = format!("https://a.4cdn.org/{board}/threads.json");
        let reply: Reply<Vec<BaseThread>> = client.fetch_json(&url, None).await?;
        let last_modified = reply
            .last_modified
            .ok_or_else(|| MissingHeader(LAST_MODIFIED))?;

        let threads = reply.inner?;
        let metadata = Metadata { url, last_modified };
        Ok(Self { threads, metadata })
    }

    /// Refreshes the contents of the `ThreadList`.
    ///
    /// This method updates the threads and associated metadata.
    /// Using this method will overwrite the currently held data.
    ///
    /// # Errors
    ///
    /// This function will return an error if the client fails to fetch
    /// the updated data.
    pub async fn update(&mut self, client: &Client) -> Result<()> {
        let reply: Reply<Vec<BaseThread>> = client
            .fetch_json(self.metadata.url(), Some(&self.metadata.last_modified))
            .await?;

        match reply.inner {
            Ok(i) => self.threads = i,
            Err(Error::NotModified) => {}
            Err(x) => return Err(x),
        }
        if let Some(lm) = reply.last_modified {
            log::debug!("updating last modified");
            self.metadata.last_modified = lm;
        }
        Ok(())
    }
}

impl std::ops::Deref for ThreadList {
    type Target = Vec<BaseThread>;

    fn deref(&self) -> &Self::Target {
        &self.threads
    }
}

/// Represents a page of threads.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseThread {
    /// Page number that the threads are listed on.
    page: u32,
    /// List of threads on the page.
    threads: Vec<ThreadAttributes>,
}

impl BaseThread {
    /// Returns the page number that the threads are listed on.
    pub fn page(&self) -> u32 {
        self.page
    }
    /// Returns an array of threads on the page.
    pub fn threads(&self) -> &[ThreadAttributes] {
        &self.threads
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
/// Attributes the attributes of a thread.
pub struct ThreadAttributes {
    /// Original post number (OP ID) of the thread.
    no: u32,
    /// UNIX timestamp marking the last time the thread was modified.
    last_modified: u64,
    /// Count of the number of replies in the thread.
    replies: u32,
}

impl ThreadAttributes {
    /// Returns the original post number (OP ID) of the thread.
    pub fn no(&self) -> u32 {
        self.no
    }
    /// Returns UNIX timestamp marking the last time the thread was modified.
    pub fn last_modified(&self) -> u64 {
        self.last_modified
    }
    /// Return the number of replies in the thread.
    pub fn replies(&self) -> u32 {
        self.replies
    }
}
