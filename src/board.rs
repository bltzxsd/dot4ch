//! A cache of the entire board.
//!
//! It is recommended to build it once and update only after long intervals.
//!
//! Please rely on updating induidual threads if you do not need *all* posts from a thead.
//!
//! This is an expensive and time consuming build for the first time however.

use crate::{thread::Thread, threadlist::Catalog, Update};
use async_trait::async_trait;
use log::debug;
use std::{collections::HashMap, sync::Arc};
type CrateClient = Arc<tokio::sync::Mutex<crate::Client>>;

#[derive(Debug)]
/// Holds an abstraction over `HashMap<u32, Thread>` which can be used to any post with a Post number.
pub struct Board {
    pub threads: HashMap<u32, Thread>,
    pub board: String,
}

impl Board {
    /// Returns an entire board containing *all* of its posts at one point.
    ///
    /// This is an expensive one time operation.
    ///
    /// # Time
    ///
    /// A typical board of ~150 threads takes 5+ minutes to cache.
    ///
    /// It is advised to build this only once due to its long wait times caused by API cooldowns.
    pub async fn build(client: &mut CrateClient, board: &str) -> crate::Result<Self> {
        let catalog = Catalog::new(client, board).await?;
        let ids: Vec<_> = catalog
            .all_pages()
            .into_iter()
            .flat_map(|f| f.threads())
            .map(|thread| thread.id())
            .collect();
        debug!("Threads: {:#?}\nNumber of threads: {}", ids, ids.len() - 1);
        let mut threads = vec![];
        for id in &ids {
            threads.push(Thread::new(client, board, *id).await?);
        }

        let threads: Vec<_> = threads.into_iter().zip(ids).collect();
        let mut id_thread_zip = HashMap::new();
        for (thread, num) in threads {
            id_thread_zip.insert(num, thread);
        }
        Ok(Self {
            threads: id_thread_zip,
            board: board.to_string(),
        })
    }

    /// Returns a specific Thread from the Board cache.
    pub fn get(self, k: u32) -> Option<Thread> {
        let f = self.threads.get(&k);
        f.cloned()
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
    type Output = Board;
    /// Returns an updated board.
    ///
    /// It is recommended to call this infrequently due to API calls having cooldowns.
    ///
    /// Uses `If-Modified-Since` header internally.
    async fn update(mut self, client: &mut CrateClient) -> crate::Result<Self::Output> {
        // get the ID's of all the threads we need
        let catalog = Catalog::new(client, &self.board).await?;
        let ids: Vec<_> = catalog
            .all_pages()
            .into_iter()
            .flat_map(|f| f.threads())
            .map(|thread| thread.id())
            .collect();
        let mut threads = vec![];

        for (_, thread) in self.threads.into_iter() {
            // update all threads with the ID
            threads.push(thread.update(client).await?);
        }
        let mut id_thread_zip = HashMap::new();
        let threads: Vec<_> = threads.into_iter().zip(ids).collect();
        for (thread, num) in threads {
            id_thread_zip.insert(num, thread);
        }
        Ok(Self {
            threads: id_thread_zip,
            board: self.board,
        })
    }
}
