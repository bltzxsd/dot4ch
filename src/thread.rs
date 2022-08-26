//! Contains information about a 4chan thread.
//!
//! This is documented as `thread.json` in the
//! [4chan API Repository](<https://github.com/4chan/4chan-API/blob/master/pages/Threads.md>)
//!
//! This contains all the replies from the given thread.
//!

use super::{post::Post, Result};
use crate::{board::Board, error::DotError, Dot4chClient, IfModifiedSince, Procedures, Update};
use async_trait::async_trait;
use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use log::debug;
use reqwest::{header::IF_MODIFIED_SINCE, Response, StatusCode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::time;

/// The main end user interface to the 4chan thread API.
///
/// Contains data about a chan thread.
#[derive(Debug, Clone)]
pub struct Thread {
    /// The Original Post
    op: Post,
    /// The board the post is on
    board: String,
    /// The number of replies
    replies_no: usize,
    /// The latest reply
    last_reply: Option<u32>,
    /// All the posts in the thread
    all_replies: Vec<Post>,
    /// When the thread was archived
    archive_time: Option<NaiveDateTime>,
    /// Thread archival status
    archived: bool,
    /// Last time the thread was requested.
    last_update: Option<DateTime<Utc>>,
    /// the client
    client: Dot4chClient,
}

#[cfg(feature = "display")]
impl Display for Thread {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let fmt = format!(
            "OP ID: {}\nBoard: /{}/\nNumber of Replies: {}\nArchived: {}\n",
            self.op.id(),
            self.board,
            self.replies_no,
            self.archived
        );
        write!(f, "{}", fmt)
    }
}

#[async_trait(?Send)]
impl IfModifiedSince for Thread {
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
impl Update for Thread {
    type Output = Self;
    /// Returns the updated 4chan thread.
    ///
    /// `update()` respects
    /// 4chan's 10 seconds between each chan thread call.
    async fn update(mut self) -> Result<Self> {
        if self.archived {
            let archival_time = self.archive_time.ok_or_else(|| DotError::Thread("cannot get archival time".to_string()))?;
            let time = archival_time.format("%a, %d %b %Y %T").to_string();
            return Err(DotError::Archived { time, thread: self.op().id()})
        }

        self.refresh_time().await?;

        let header = crate::header(&self.client).await;
        let response = Self::fetch(&self.client, &self.thread_url(), &header).await?;
        self.client.lock().await.last_checked = Utc::now();

        let mut thread = self.fetch_status(response).await?;

        thread.update_time();

        thread.client.lock().await.last_checked = Utc::now();
        Ok(thread)
    }
}

#[async_trait(?Send)]
impl Procedures for Thread {
    type Output = Self;
    async fn refresh_time(&mut self) -> Result<()> {
        if let Some(time) = self.last_update {
            let curr = Utc::now().signed_duration_since(time);
            if curr < Duration::seconds(10) {
                let remaining = 10 - curr.num_seconds();
                debug!(
                    "Updating Thread too quickly! Waiting for {} seconds",
                    remaining
                );

                // None check can be bypassed since 10 - curr will usually always be a Some value.
                if let Some(time) = Duration::seconds(10).checked_sub(&curr) {
                    time::sleep(
                        time.to_std()
                            .expect("could not change chrono time to stdtime"),
                    )
                    .await;
                }
            }
        };
        Ok(())
    }

    /// Checks the status of a [`Response`] and generates a new thread if needed.
    async fn fetch_status(self, response: Response) -> crate::Result<Self::Output> {
        match response.status() {
            StatusCode::OK => return self.from_response(response).await,
            StatusCode::NOT_MODIFIED => {
                let mut thread = self;
                thread.last_update = Some(Utc::now());
                return Ok(thread);
            }
            other_resp => return Err(DotError::Update(format!("unexpected response code {}", other_resp)))?,
        }
    }

    /// Converts the `Response` into a `Thread`
    async fn from_response(self, response: Response) -> crate::Result<Self::Output> {
        // Note: into json is ok here since StatusCode is OK
        // and any further errors will be from Parsing JSON
        let thread_data = response.json::<DeserializedThread>().await?.posts;

        Ok(Self {
            op: thread_data.first().expect("No OP found").clone(),
            board: self.board().to_string(),
            replies_no: thread_data.len() - 1_usize,
            last_reply: thread_data.last().map(Post::id),
            all_replies: thread_data.iter().skip(1).cloned().collect(),
            archive_time: thread_data
                .first()
                .map(|data| NaiveDateTime::from_timestamp(data.archived_on(), 0)),
            archived: thread_data.first().expect("No OP found.").archived(),
            last_update: Some(Utc::now()),
            client: self.client.clone(),
        })
    }
}

impl Thread {
    /// Create a new [`Thread`].
    ///
    /// **Requires the Board to be a valid 4chan board
    /// and the Post ID to be a valid 4chan OP Post ID.**
    ///
    /// # Errors
    ///
    /// This function will panic if it does not find an OP for the thread.
    pub async fn new(client: &Dot4chClient, board: &str, post_id: u32) -> Result<Self> {
        let thread_data = thread_deserializer(client, board, post_id).await?.posts;
        let op = { thread_data.first().expect("NO OP FOUND").clone() };
        let archived = op.archived();
        let last_reply = thread_data.last().map(Post::id);

        let archive_time = if archived {
            let secs = thread_data.first().expect("NO OP FOUND").archived_on();
            Some(NaiveDateTime::from_timestamp(secs, 0))
        } else {
            None
        };

        Ok(Self {
            op,
            board: board.to_string(),
            replies_no: thread_data.len() - 1_usize,
            last_reply,
            all_replies: thread_data.iter().skip(1).cloned().collect(),
            archive_time,
            archived,
            last_update: None,
            client: client.clone(),
        })
    }

    /// Find an post with an ID
    ///
    /// Returns the first element of
    pub fn find(&self, id: u32) -> Option<&Post> {
        self.all_replies.iter().find(|post| post.id() == id)
    }

    /// Updates the time when the last GET was performed
    pub fn update_time(&mut self) {
        self.last_update = Some(Utc::now());
    }

    /// Returns a reference the original post of thread
    pub fn op(&self) -> &Post {
        &self.op
    }

    /// Return the last post from a thread
    pub fn last_post(&self) -> Option<&Post> {
        self.all_replies.last()
    }

    /// Return the ID of the last reply
    pub fn last_reply(&self) -> Option<u32> {
        self.last_reply
    }

    /// Return the number of replies 
    pub fn replies_no(&self) -> usize {
        self.replies_no
    }

    /// Return the name of the board
    pub fn board(&self) -> &str {
        &self.board
    }

    /// Return the API URL of a thread.
    pub fn thread_url(&self) -> String {
        format!(
            "https://a.4cdn.org/{}/thread/{}.json",
            self.board,
            self.op().id()
        )
    }

    /// Convert one [`Thread`] to a [`Board`]
    pub fn into_board(self) -> Board {
        let mut hash = HashMap::new();
        let num = &self.op.id();
        let client = self.client.clone();
        let board = self.board.to_string();
        hash.insert(*num, self);
        Board {
            threads: hash,
            board,
            client,
        }
    }
}

impl From<Thread> for Board {
    fn from(thread: Thread) -> Self {
        let mut hash = HashMap::new();
        let num = thread.op.id();
        let client = thread.client.clone();
        let board = thread.board().to_string();
        hash.insert(num, thread);
        Board {
            threads: hash,
            board,
            client,
        }
    }
}

impl<Idx> std::ops::Index<Idx> for Thread
where
    Idx: std::slice::SliceIndex<[Post]>,
{
    type Output = Idx::Output;

    fn index(&self, index: Idx) -> &Self::Output {
        &self.all_replies[index]
    }
}

/// The intermediate representation(?) of a thread.
///
/// You do not need to contruct this in most cases since it will be handled internally by
/// `from_deserialize()`
///
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct DeserializedThread {
    /// A vector of posts. Used internally.
    posts: Vec<Post>,
}

/// Converts 4chan thread JSON to [`DeserializedThread`].
///
/// This is a helper function to `from_deserialized()`
///
/// # Errors
///
/// Returns an error if the given thread is not found
async fn thread_deserializer(
    client: &Dot4chClient,
    board: &str,
    post_num: u32,
) -> Result<DeserializedThread> {
    let rq = format!("https://a.4cdn.org/{}/thread/{}.json", board, post_num);
    let req = client.lock().await.get(&rq).await?;

    req.error_for_status_ref()?;

    let req = req.json::<DeserializedThread>().await?;
    debug!("Deserialized Post: {}", post_num);
    Ok(req)
}
