//! Contains information about a 4chan thread.
//!

use crate::{Dot4chClient, IfModifiedSince, Update};
use async_trait::async_trait;

use super::{post::Post, Result};
use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use log::{debug, info};
use reqwest::{header::IF_MODIFIED_SINCE, Response, StatusCode};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
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
}

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
    ) -> std::result::Result<Response, reqwest::Error> {
        client
            .lock()
            .await
            .req_client()
            .get(url)
            .header(IF_MODIFIED_SINCE, header)
            .send()
            .await
    }
}

#[async_trait(?Send)]
impl Update for Thread {
    type Output = Self;
    /// Returns the updated 4chan thread.
    ///
    /// `update()` respects
    /// 4chan's 10 seconds between each chan thread call.
    async fn update(mut self, client: &Dot4chClient) -> Result<Self> {
        if self.archived {
            let archival_time = match self.archive_time {
                Some(time) => time.format("%a, %d %b %Y %T"),
                None => return Err(From::from("Archival time was not found on thread.")),
            };
            let formatted = format!(
                "Thread: [{}] got archived at: {}",
                self.op().id(),
                archival_time
            );
            return Err(From::from(formatted));
        }

        // Fetch requests staggered every 10 seconds
        if let Some(time) = self.last_update {
            let curr = Utc::now().signed_duration_since(time);
            if curr < Duration::seconds(10) {
                info!("Tried updating Thread within 10 seconds. Sleeping until cooldown...");
                let dur = Duration::seconds(10).checked_sub(&curr);
                match dur {
                    // unwrap is fine here since if its < 0, then we already match with None and return Error.
                    Some(time) => time::sleep(time.to_std().expect("value could not fit")).await,
                    None => return Err(From::from("Could not subtract time in Thread")),
                }
            }
        }

        // If-Modified-Since: Wed, 21 Oct 2015 07:28:00 GMT
        // strftime fmt:      %a , %d %b  %Y   %T       GMT
        let updated_thread = {
            let header = crate::header(client).await;
            let response = Self::fetch(client, &self.thread_url(), &header).await?;
            client.lock().await.last_checked = Utc::now();

            match response.status() {
                StatusCode::OK => {
                    let thread_data = response.json::<DeserializedThread>().await?.posts;

                    let archive_time = thread_data.first().and_then(|data| {
                        if data.archived() {
                            Some(NaiveDateTime::from_timestamp(data.archived_on(), 0))
                        } else {
                            None
                        }
                    });

                    let last_reply = thread_data.last().map(Post::id);
                    let archived = match thread_data.first() {
                        Some(boolean) => boolean.archived(),
                        None => return Err(From::from("First post was not found.")),
                    };

                    Self {
                        op: thread_data.first().expect("NO OP FOUND").clone(),
                        board: self.board().to_string(),
                        replies_no: thread_data.len() - 1_usize,
                        last_reply,
                        all_replies: thread_data.iter().skip(1).cloned().collect(),
                        archive_time,
                        archived,
                        last_update: None,
                    }
                }

                StatusCode::NOT_MODIFIED => self.clone(),

                unexpected_resp => {
                    return Err(From::from(format!(
                        "Got unexpected StatusCode {} from Thread::update()",
                        unexpected_resp
                    )))
                }
            }
        };
        let _ = self.update_time();
        debug!(
            "Changed last updated time to be: {:?}",
            // This should never fail.
            self.last_update.expect("last update debug log failed")
        );

        client.lock().await.last_checked = Utc::now();
        Ok(updated_thread)
    }
}

impl Thread {
    /// Get the data from `DeserializedThread` struct
    /// and convert it to a `Thread`.
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
        let archive_time = if archived {
            let timestamp = thread_data.first().expect("First not found.").archived_on();
            let naive = NaiveDateTime::from_timestamp(timestamp, 0);
            Some(naive)
        } else {
            None
        };
        let last_reply = thread_data.last().map(Post::id);

        Ok(Self {
            op,
            board: board.to_string(),
            replies_no: thread_data.len() - 1_usize,
            last_reply,
            all_replies: thread_data.iter().skip(1).cloned().collect(),
            archive_time,
            archived,
            last_update: None,
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

    /// Returns a specific post from a thread by its index in the thread.
    ///
    /// Returns `None` if it does not exist.
    pub fn get(&self, idx: usize) -> Option<&Post> {
        self.all_replies.get(idx)
    }

    /// Return the last post from a thread
    pub fn last_post(&self) -> Option<&Post> {
        self.all_replies.last()
    }

    /// Return the current board
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

/// Converts 4chan thread JSON to `DeserializedThread`.
///
/// This is a helper function to `from_deserialized()`
///
/// # Errors
///
/// Throws an error if the given thread is not found
async fn thread_deserializer(
    client: &Dot4chClient,
    board: &str,
    post_num: u32,
) -> Result<DeserializedThread> {
    let rq = format!("https://a.4cdn.org/{}/thread/{}.json", board, post_num);
    let req = client
        .lock()
        .await
        .get(&rq)
        .await?
        .json::<DeserializedThread>()
        .await?;
    debug!("Deserialized Post: {}", post_num);
    Ok(req)
}
