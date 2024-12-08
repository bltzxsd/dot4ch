use std::time::Duration;

use crate::{
    client::Reply,
    error::Error::{self, MissingHeader},
    models::maybe_de_bool,
    models::{macros::str_opt_ref, Metadata},
    result::Result,
    Client,
};
use reqwest::header::LAST_MODIFIED;
use serde::{Deserialize, Serialize};
use tokio::time::Instant;

/// A collection of [`Post`]s representing a 4chan thread.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thread {
    posts: Vec<Post>,
    #[serde(skip)]
    pub(crate) metadata: Metadata,
    #[serde(skip)]
    last_update: Option<Instant>,
}

impl Thread {
    /// Constructs a `Thread` with a valid OP ID.
    ///
    /// # Errors
    ///
    /// This function will return an error if the client fails to fetch the data,
    /// or if the board or OP ID does not exist,
    /// or if necessary headers/content is missing from the response.
    pub async fn new(client: &Client, board: &str, op_id: u32) -> Result<Self> {
        let url = format!("https://a.4cdn.org/{board}/thread/{op_id}.json");
        let reply: Reply<Thread> = client.fetch_json(&url, None).await?;
        let last_modified = reply
            .last_modified
            .ok_or_else(|| MissingHeader(LAST_MODIFIED))?;

        let mut thread = reply.inner?;
        let metadata = Metadata { url, last_modified };
        thread.metadata = metadata;
        Ok(thread)
    }

    /// Refreshes the contents of the `Thread`.
    ///
    /// This method updates the thread and associated metadata.
    /// Using this method will overwrite the currently held data.
    ///
    /// # Rate Limits
    ///
    /// All threads have a separate rate limit of 10 seconds per update
    /// in addition to global rate limits.
    /// This rate limit is unique to each thread and will cause the task
    /// to sleep if called too frequently.
    ///
    /// # Errors
    ///
    /// This function will return an error if the client fails to fetch
    /// the updated data.
    pub async fn update(&mut self, client: &Client) -> Result<()> {
        if let Some(last_update) = self.last_update {
            let elapsed = last_update.elapsed();
            if elapsed < Duration::from_secs(10) {
                log::debug!("updating too often! rate-limiting..");
                let wait_time = Duration::from_secs(10) - elapsed;
                tokio::time::sleep(wait_time).await;
            }
        }

        let reply: Reply<Thread> = client
            .fetch_json(self.metadata.url(), Some(&self.metadata.last_modified))
            .await?;

        self.last_update = Some(Instant::now());

        match reply.inner {
            Ok(i) => self.posts = i.posts,
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

impl std::ops::Deref for Thread {
    type Target = Vec<Post>;

    fn deref(&self) -> &Self::Target {
        &self.posts
    }
}

/// Represents a post on a board, including its metadata, content, and attachments (if any).
/// This struct maps to the fields referenced in the API documentation for a post.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    /// The numeric post ID.
    no: u32,

    /// For replies: the ID of the thread being replied to. For OP posts: this value is 0.
    resto: u32,

    /// Whether the thread is stickied (pinned to the top of the page). Present only for OP posts.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "maybe_de_bool"
    )]
    sticky: Option<bool>,

    /// Whether the thread is closed to replies. Present only for OP posts.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "maybe_de_bool"
    )]
    closed: Option<bool>,

    /// Formatted string representing the time of post creation (e.g., `MM/DD/YY(Day)HH:MM`).
    /// Uses the EST/EDT timezone.
    now: String,

    /// UNIX timestamp (seconds since epoch) indicating when the post was created.
    time: u64,

    /// Name of the user who posted. Defaults to `Anonymous`.
    name: String,

    /// The user's tripcode, if included in the post (e.g., `!tripcode` or `!!securetripcode`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    trip: Option<String>,

    /// The poster's ID, if one was included with the post.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    id: Option<String>,

    /// The capcode identifier used for this post (e.g., `mod`, `admin`, `developer`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    capcode: Option<String>,

    /// The ISO 3166-1 alpha-2 country code for the poster, if country flags are enabled.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    country: Option<String>,

    /// The name of the poster's country, if country flags are enabled.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    country_name: Option<String>,

    /// The board flag code for the poster, if board flags are enabled.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    board_flag: Option<String>,

    /// The name of the board flag for the poster, if board flags are enabled.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    flag_name: Option<String>,

    /// The subject of the OP post, if one was provided.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    sub: Option<String>,

    /// The content of the post comment, in HTML-escaped format.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    com: Option<String>,

    /// UNIX timestamp (including microseconds) indicating when an image attachment was uploaded.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    tim: Option<u64>,

    /// The filename of the image as it appeared on the poster's device.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    filename: Option<String>,

    /// The file type of the image attachment (e.g., `.jpg`, `.png`, `.gif`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    ext: Option<String>,

    /// The size of the uploaded file, in bytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    fsize: Option<u32>,

    /// Base64-encoded MD5 hash of the file (24 characters).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    md5: Option<String>,

    /// The width (in pixels) of the uploaded image.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    w: Option<u32>,

    /// The height (in pixels) of the uploaded image.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    h: Option<u32>,

    /// The width (in pixels) of the image thumbnail.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    tn_w: Option<u32>,

    /// The height (in pixels) of the image thumbnail.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    tn_h: Option<u32>,

    /// Whether the file in this post was deleted.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "maybe_de_bool"
    )]
    filedeleted: Option<bool>,

    /// Whether the file in this post was marked as a spoiler.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "maybe_de_bool"
    )]
    spoiler: Option<bool>,

    /// The custom spoiler ID (allowed range: `1-10`) for this post, if applicable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    custom_spoiler: Option<u32>,

    /// Total number of replies to the thread, applicable to OP posts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    replies: Option<u32>,

    /// Total number of image replies to the thread, applicable to OP posts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    images: Option<u32>,

    /// Indicates whether the thread has reached its bump limit (present for OP threads).
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "maybe_de_bool"
    )]
    bumplimit: Option<bool>,

    /// Indicates whether the thread has reached its image limit (present for OP threads).
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "maybe_de_bool"
    )]
    imagelimit: Option<bool>,

    /// The category of `.swf` files (e.g., `Game`, `Loop`) applicable to `/f/` boards.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    tag: Option<String>,

    /// The (SEO-friendly) URL slug for this thread, applicable to OP threads.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    semantic_url: Option<String>,

    /// The year the user bought a 4chan pass, if specified.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    since4pass: Option<u32>,

    /// The number of unique posters in a thread, visible for non-archived threads.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    unique_ips: Option<u32>,

    /// Whether the thread has a mobile-optimized image.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "maybe_de_bool"
    )]
    m_img: Option<bool>,

    /// Whether the thread has been archived.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "maybe_de_bool"
    )]
    archived: Option<bool>,

    /// UNIX timestamp (seconds since epoch) indicating when the thread was archived.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    archived_on: Option<u64>,
}

impl Post {
    /// Returns the numeric post ID.
    pub fn no(&self) -> u32 {
        self.no
    }

    /// Returns the thread ID (or `0` if the post is OP).
    pub fn resto(&self) -> u32 {
        self.resto
    }

    /// Returns whether the thread is stickied to the top of the page.
    pub fn sticky(&self) -> Option<bool> {
        self.sticky
    }

    /// Returns whether the thread is closed to replies.
    pub fn closed(&self) -> Option<bool> {
        self.closed
    }

    /// Returns the formatted creation time of the post.
    pub fn now(&self) -> &str {
        &self.now
    }

    /// Returns the UNIX timestamp when the post was created.
    pub fn time(&self) -> u64 {
        self.time
    }

    /// Returns the name of the user who posted.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the user's tripcode (if present).
    pub fn trip(&self) -> Option<&str> {
        str_opt_ref!(self.trip)
    }

    /// Returns the poster's ID (if present).
    pub fn id(&self) -> Option<&str> {
        str_opt_ref!(self.id)
    }

    /// Returns the capcode identifier for the post (if present).
    pub fn capcode(&self) -> Option<&str> {
        str_opt_ref!(self.capcode)
    }

    /// Returns the ISO 3166-1 alpha-2 country code for the poster (if present).
    pub fn country(&self) -> Option<&str> {
        str_opt_ref!(self.country)
    }

    /// Returns the name of the poster's country (if present).
    pub fn country_name(&self) -> Option<&str> {
        str_opt_ref!(self.country_name)
    }

    /// Returns the board flag code for the poster (if present).
    pub fn board_flag(&self) -> Option<&str> {
        str_opt_ref!(self.board_flag)
    }

    /// Returns the name of the board flag for the poster (if present).
    pub fn flag_name(&self) -> Option<&str> {
        str_opt_ref!(self.flag_name)
    }

    /// Returns the subject for an OP post (if provided).
    pub fn sub(&self) -> Option<&str> {
        str_opt_ref!(self.sub)
    }

    /// Returns the comment content of the post (if provided).
    pub fn com(&self) -> Option<&str> {
        str_opt_ref!(self.com)
    }

    /// Returns the UNIX timestamp of the time the image was uploaded (if present).
    pub fn tim(&self) -> Option<u64> {
        self.tim
    }

    /// Returns the filename of the uploaded image (if present).
    pub fn filename(&self) -> Option<&str> {
        str_opt_ref!(self.filename)
    }

    /// Returns the file extension of the uploaded image (if present).
    pub fn ext(&self) -> Option<&str> {
        str_opt_ref!(self.ext)
    }

    /// Returns the size of the uploaded file in bytes (if present).
    pub fn fsize(&self) -> Option<u32> {
        self.fsize
    }

    /// Returns the MD5 hash of the uploaded file (if present).
    pub fn md5(&self) -> Option<&str> {
        str_opt_ref!(self.md5)
    }

    /// Returns the width of the uploaded image (if present).
    pub fn w(&self) -> Option<u32> {
        self.w
    }

    /// Returns the height of the uploaded image (if present).
    pub fn h(&self) -> Option<u32> {
        self.h
    }

    /// Returns the thumbnail width of the uploaded image (if present).
    pub fn tn_w(&self) -> Option<u32> {
        self.tn_w
    }

    /// Returns the thumbnail height of the uploaded image (if present).
    pub fn tn_h(&self) -> Option<u32> {
        self.tn_h
    }

    /// Returns whether the file was deleted (if set).
    pub fn filedeleted(&self) -> Option<bool> {
        self.filedeleted
    }

    /// Returns whether the image in the post was marked as a spoiler.
    pub fn spoiler(&self) -> Option<bool> {
        self.spoiler
    }

    /// Returns the custom spoiler ID for the image (if set).
    pub fn custom_spoiler(&self) -> Option<u32> {
        self.custom_spoiler
    }

    /// Returns the total number of replies to the thread (if present; OP only).
    pub fn replies(&self) -> Option<u32> {
        self.replies
    }

    /// Returns the total number of image replies to the thread (if present; OP only).
    pub fn images(&self) -> Option<u32> {
        self.images
    }

    /// Returns whether the thread has reached the bump limit.
    pub fn bumplimit(&self) -> Option<bool> {
        self.bumplimit
    }

    /// Returns whether the thread has reached the image limit.
    pub fn imagelimit(&self) -> Option<bool> {
        self.imagelimit
    }

    /// Returns the `.swf` upload tag (e.g., `Game`, `Loop`) for `/f/` boards.
    pub fn tag(&self) -> Option<&str> {
        str_opt_ref!(self.tag)
    }

    /// Returns the SEO URL slug for a thread (if present; OP only).
    pub fn semantic_url(&self) -> Option<&str> {
        str_opt_ref!(self.semantic_url)
    }

    /// Returns the year the poster purchased a 4chan pass (if set).
    pub fn since4pass(&self) -> Option<u32> {
        self.since4pass
    }

    /// Returns the number of unique IPs in a thread (if not archived; OP only).
    pub fn unique_ips(&self) -> Option<u32> {
        self.unique_ips
    }

    /// Returns whether a mobile-optimized image is available for the post.
    pub fn m_img(&self) -> Option<bool> {
        self.m_img
    }

    /// Returns whether the thread is archived.
    pub fn archived(&self) -> Option<bool> {
        self.archived
    }

    /// Returns the UNIX timestamp for when the thread was archived (if set).
    pub fn archived_on(&self) -> Option<u64> {
        self.archived_on
    }
}
