//! A cache of the entire board.
//!
//! The [`Board`] is a comprehensive collection of all threads and posts on a board.
//!
//! Please rely on updating individual [`Thread`]s you do not need *all* posts from a thread.
//!
//! # Time
//!
//! Due to API constraints, a board is built rather slowly.
//!
//! It is recommended to update a board in intervals of no less than than 10 minutes.
//!
//! # Example: Building a board and updating it
//! ```rust
//! # use dot4ch::{Client, Update, board::Board};
//! # async fn something() -> Result<(()), anyhow::Error> {
//! # // Making a client
//! let client = Client::new();
//!
//! // Building the /g/ board
//! let board = Board::build(&client, "g").await?;
//!
//! /* Do something with the board */
//!
//! // After a long interval, we update it.
//! let g = board.update().await?;
//! println!("{:#?}", g);
//! # Ok(())
//! # }
//! ```

use crate::{thread::Thread, threadlist::Catalog, Dot4chClient, Update};
use async_trait::async_trait;
use log::info;

use std::{
    collections::HashMap,
    io::{self, Write},
};

#[derive(Debug)]
/// Holds an abstraction over [`HashMap<u32, Thread>`].
pub struct Board {
    /// A HashMap of Thread and their ID's
    pub threads: HashMap<u32, Thread>,
    /// The board on this instance of board is based.
    pub(crate) board: String,
    /// the client
    pub(crate) client: Dot4chClient,
}

impl Board {
    /// Returns an entire board containing *all* of it's posts.
    ///
    /// # Time
    ///
    /// A typical board of ~150 threads takes 5+ minutes to cache.
    ///
    /// It is advised to build this only once due to its long wait times caused by API cooldowns.
    ///
    /// # Errors
    ///
    /// This function will return an error if the request to get a new [`Catalog`] fails.
    pub async fn build(client: &Dot4chClient, board: &str) -> crate::Result<Self> {
        writeln!(io::stdout(), "Building Board! Please wait.")?;
        let catalog = Catalog::new(client, board).await?;
        let ids: Vec<_> = catalog
            .all_pages()
            .into_iter()
            .flat_map(crate::threadlist::Page::threads)
            .map(|thread| thread.id())
            .collect();

        info!("Number of threads: {}", ids.len());
        let mut threads = vec![];
        for (idx, id) in ids.iter().enumerate() {
            threads.push(Thread::new(client, board, *id).await?);
            info!("Pushed Thread: {}/{}", idx + 1, ids.len())
        }

        let threads: Vec<_> = threads.into_iter().zip(ids).collect();
        let mut id_thread_zip = HashMap::new();
        for (thread, num) in threads {
            id_thread_zip.insert(num, thread);
        }
        Ok(Self {
            threads: id_thread_zip,
            board: board.to_string(),
            client: client.clone(),
        })
    }

    /// Returns a specific Thread from the Board cache.
    pub fn get(&self, k: u32) -> Option<&'_ Thread> {
        self.threads.get(&k)
    }

    /// Inserts a new thread into a cache.
    ///
    /// If a thread already exists, it updates the thread
    /// while retaining the post number and returns the old thread.
    pub fn insert(&mut self, id: u32, thread: Thread) -> Option<Thread> {
        self.threads.insert(id, thread)
    }

    /// Returns the board of the cache
    pub fn board(&self) -> &str {
        &self.board
    }
}

#[async_trait(?Send)]
impl Update for Board {
    type Output = Self;
    /// Returns an updated board.
    ///
    /// It is recommended to call this infrequently due to API calls having cooldowns.
    ///
    /// Uses `If-Modified-Since` header internally.
    async fn update(mut self) -> crate::Result<Self::Output> {
        // get the ID's of all the threads we need
        // we have to call this again because there might be new thread that need to be added.
        writeln!(io::stdout(), "Updating Board. Please wait..")?;
        let catalog = Catalog::new(&self.client, &self.board).await?;
        let ids: Vec<_> = catalog
            .all_pages()
            .into_iter()
            .flat_map(crate::threadlist::Page::threads)
            .map(|thread| thread.id())
            .collect();

        let mut threads = vec![];
        for (num, (id, thread)) in self.threads.into_iter().enumerate() {
            // update all threads with the ID
            threads.push(thread.update().await?);
            info!(
                "Updating thread: {}\t Threads updated: {}/{}",
                id,
                (num + 1),
                &ids.len()
            );
        }

        let mut id_thread_zip = HashMap::new();
        let threads: Vec<_> = threads.into_iter().zip(ids).collect();
        for (thread, num) in threads {
            id_thread_zip.insert(num, thread);
        }
        writeln!(io::stdout(), "Finished updating threads!")?;
        Ok(Self {
            threads: id_thread_zip,
            board: self.board,
            client: self.client,
        })
    }
}
