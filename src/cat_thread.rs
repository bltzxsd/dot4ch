use serde::{Deserialize, Serialize};
use std::{ops::Index, slice::SliceIndex};

use crate::{thread::Thread, Dot4chClient};

/// Contains some metadata about a catalog thread.
///
/// Usually used in the context of a [`Page`]
#[derive(Debug, Serialize, Deserialize, Default, Eq, PartialEq, Clone, Copy)]
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
    /// This function will fail if the request to fetch the [`Thread`] does not succeed.
    pub async fn to_thread(self, client: &Dot4chClient, board: &str) -> crate::Result<Thread> {
        Thread::new(client, board, self.no).await
    }
}

#[cfg(feature = "cat_thread_display")]
impl Display for CatalogThread {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let g = chrono::NaiveDateTime::from_timestamp(self.last_modified, 0);
        let fmt = format!(
            "\n\tThread ID: {} | Last Modified: {} | Number of Replies: {}",
            self.no, g, self.replies
        );
        write!(f, "{}", fmt)
    }
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
/// A Page in the catalog.
/// Pages contain their own number and a vector of [`CatalogThread`]s
///
/// This is usually the intermediate between a [`Catalog`] and a [`CatalogThread`]
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

#[cfg(feature = "page_display")]
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

impl<Idx> Index<Idx> for Page
where
    Idx: SliceIndex<[CatalogThread]>,
{
    type Output = Idx::Output;

    fn index(&self, index: Idx) -> &Self::Output {
        &self.threads[index]
    }
}
