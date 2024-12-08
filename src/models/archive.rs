use reqwest::header::LAST_MODIFIED;
use serde::{Deserialize, Serialize};

use crate::{
    client::Reply,
    error::Error::{self, MissingHeader},
    models::Metadata,
    result::Result,
    Client,
};

/// Represents a collection of archived thread IDs for a specific 4chan board.
///
/// The `Archive` struct holds thread IDs and their metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Archive {
    fields: Vec<u32>,
    #[serde(skip)]
    pub(crate) metadata: Metadata,
}

impl std::ops::Deref for Archive {
    type Target = Vec<u32>;

    fn deref(&self) -> &Self::Target {
        &self.fields
    }
}

impl Archive {
    /// Constructs a new `Archive` for a specified board.
    ///
    /// It initializes the archive by fetching data from the given board's archive JSON.
    ///
    /// # Errors
    ///
    /// This function returns an error if the fetch operation fails or if expected headers are absent.
    pub async fn new(client: &Client, board: &str) -> Result<Self> {
        let url = format!("https://a.4cdn.org/{board}/archive.json");
        let reply: Reply<Vec<u32>> = client.fetch_json(&url, None).await?;
        let last_modified = reply
            .last_modified
            .ok_or_else(|| MissingHeader(LAST_MODIFIED))?;

        let fields = reply.inner?;
        let metadata = Metadata { url, last_modified };
        Ok(Self { fields, metadata })
    }

    /// Updates the `Archive` with the latest set of thread IDs.
    ///
    /// Refreshes archived thread IDs using the stored URL and metadata, overwriting the current state.
    ///
    /// # Errors
    ///
    /// Will return an error if fetching fails or if necessary data is missing.
    pub async fn update(&mut self, client: &Client) -> Result<()> {
        let reply: Reply<Vec<u32>> = client
            .fetch_json(self.metadata.url(), Some(&self.metadata.last_modified))
            .await?;

        match reply.inner {
            Ok(i) => self.fields = i,
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
