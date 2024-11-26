use std::ops::Deref;

use crate::{
    client::Reply,
    error::Error::{self, MissingHeader},
    models::{macros::str_opt_ref, maybe_de_bool, Metadata},
    result::Result,
    Client,
};
use reqwest::header::LAST_MODIFIED;
use serde::{Deserialize, Serialize};

/// Represents a catalog of threads and their attributes, organized by pages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Catalog {
    /// Vector of [`Page`]
    pages: Vec<Page>,

    #[serde(skip)]
    pub(crate) metadata: Metadata,
}

impl Deref for Catalog {
    type Target = Vec<Page>;

    fn deref(&self) -> &Self::Target {
        &self.pages
    }
}

impl Catalog {
    /// Constructs a `Catalog` containing all threads of a specified board.
    ///
    /// # Errors
    ///
    /// Returns an error if the client fails to fetch data,
    /// if the board doesn't exist, or if required headers are absent.
    pub async fn new(client: &Client, board: &str) -> Result<Self> {
        let url = format!("https://a.4cdn.org/{board}/catalog.json");
        let reply: Reply<Vec<Page>> = client.fetch_json(&url, None).await?;
        let last_modified = reply
            .last_modified
            .ok_or_else(|| MissingHeader(LAST_MODIFIED))?;

        let pages = reply.inner?;
        let metadata = Metadata { url, last_modified };
        Ok(Self { pages, metadata })
    }

    /// Updates the catalog of threads and their attributes.
    ///
    /// Retrieves new pages and threads, overwriting the existing data.
    ///
    /// # Errors
    ///
    /// Will fail if the client is unable to fetch updated data.
    pub async fn update(&mut self, client: &Client) -> Result<()> {
        let reply: Reply<Vec<Page>> = client
            .fetch_json(self.metadata.url(), Some(&self.metadata.last_modified))
            .await?;

        match reply.inner {
            Ok(i) => self.pages = i,
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

/// Represents a page within the [`Catalog`], containing multiple threads.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    /// current page number
    page: u32,
    /// threads in the current page
    threads: Vec<CatPost>,
}

impl Page {
    /// Returns the page number of the page.
    pub fn page(&self) -> u32 {
        self.page
    }

    /// Returns an array containing Catalog Posts
    pub fn threads(&self) -> &[CatPost] {
        &self.threads
    }
}

/// Contains the attributes of a post in a catalog.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CatPost {
    /// The numeric post ID
    no: i32,

    /// ID of the thread being replied to (0 for OP)
    resto: i32,

    /// 1 if the thread is stickied, not present otherwise
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "maybe_de_bool"
    )]
    sticky: Option<bool>,

    /// 1 if the thread is closed, not present otherwise
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "maybe_de_bool"
    )]
    closed: Option<bool>,

    /// Time of post creation in MM/DD/YY(Day)HH:MM(:SS) EST/EDT format
    now: String,

    /// UNIX timestamp of post creation
    time: u64,

    /// Name user posted with (defaults to "Anonymous")
    name: String,

    /// Tripcode for the post (if present)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    trip: Option<String>,

    /// Poster's ID (if present)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    id: Option<String>,

    /// Capcode for the post (if present)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    capcode: Option<String>,

    /// Poster's ISO 3166-1 alpha-2 country code (if country flags are enabled)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    country: Option<String>,

    /// Poster's country name (if country flags are enabled)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    country_name: Option<String>,

    /// OP subject text (if present)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    sub: Option<String>,

    /// Comment (HTML escaped) if present
    #[serde(default)]
    com: String,

    /// Unix timestamp of image upload (if post has attachment)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    tim: Option<i64>,

    /// Filename as it appeared on the poster's device (if post has attachment)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    filename: Option<String>,

    /// Filetype (if post has attachment)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    ext: Option<String>,

    /// Size of uploaded file in bytes (if post has attachment)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    fsize: Option<i32>,

    /// 24 character, packed base64 MD5 hash of file (if post has attachment)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    md5: Option<String>,

    /// Image width dimension (if post has attachment)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    w: Option<i32>,

    /// Image height dimension (if post has attachment)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    h: Option<i32>,

    /// Thumbnail image width dimension (if post has attachment)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    tn_w: Option<i32>,

    /// Thumbnail image height dimension (if post has attachment)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    tn_h: Option<i32>,

    /// 1 if file was deleted from post (if post had attachment)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    filedeleted: Option<i32>,

    /// 1 if image was spoilered (if post has attachment)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    spoiler: Option<i32>,

    /// Custom spoiler ID for spoilered image (if post has attachment)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    custom_spoiler: Option<i32>,

    /// Number of replies minus number of previewed replies (OP only)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    omitted_posts: Option<i32>,

    /// Number of image replies minus number of previewed image replies (OP only)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    omitted_images: Option<i32>,

    /// Total number of replies to thread (OP only)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    replies: Option<i32>,

    /// Total number of image replies to thread (OP only)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    images: Option<i32>,

    /// 1 if thread has reached bumplimit (OP only)
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "maybe_de_bool"
    )]
    bumplimit: Option<bool>,

    /// 1 if thread has reached imagelimit (OP only)
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "maybe_de_bool"
    )]
    imagelimit: Option<bool>,

    /// UNIX timestamp of last thread modification (OP only)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    last_modified: Option<u64>,

    /// Category of.swf upload (OP only, /f/ boards only)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    tag: Option<String>,

    /// SEO URL slug for thread (OP only)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    semantic_url: Option<String>,

    /// Year 4chan pass was bought (if present)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    since4pass: Option<i32>,

    /// Number of unique posters in thread (OP only)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    unique_ips: Option<i32>,

    /// 1 if mobile-optimized image exists for post
    #[serde(default, skip_serializing_if = "Option::is_none")]
    m_img: Option<bool>,

    /// JSON representation of most recent replies to thread (catalog OP only)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    last_replies: Option<Vec<CatPost>>,
}

impl CatPost {
    /// Returns the numeric post ID.
    pub fn no(&self) -> i32 {
        self.no
    }

    /// Returns the ID of the thread being replied to, or 0 for OP.
    pub fn resto(&self) -> i32 {
        self.resto
    }

    /// Returns true if the thread is stickied, or None otherwise.
    pub fn sticky(&self) -> Option<bool> {
        self.sticky
    }

    /// Returns true if the thread is closed, or None otherwise.
    pub fn closed(&self) -> Option<bool> {
        self.closed
    }

    /// Returns the time of post creation as a formatted string.
    pub fn now(&self) -> &str {
        &self.now
    }

    /// Returns the UNIX timestamp of post creation.
    pub fn time(&self) -> u64 {
        self.time
    }

    /// Returns the name user posted with.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the tripcode if present, or None otherwise.
    pub fn trip(&self) -> Option<&str> {
        str_opt_ref!(self.trip)
    }

    /// Returns the poster's ID if present, or None otherwise.
    pub fn id(&self) -> Option<&str> {
        str_opt_ref!(self.id)
    }

    /// Returns the capcode if present, or None otherwise.
    pub fn capcode(&self) -> Option<&str> {
        str_opt_ref!(self.capcode)
    }

    /// Returns the poster's country code if present, or None otherwise.
    pub fn country(&self) -> Option<&str> {
        str_opt_ref!(self.country)
    }

    /// Returns the poster's country name if present, or None otherwise.
    pub fn country_name(&self) -> Option<&str> {
        str_opt_ref!(self.country_name)
    }

    /// Returns the subject text if present, or None otherwise.
    pub fn sub(&self) -> Option<&str> {
        str_opt_ref!(self.sub)
    }

    /// Returns the comment in HTML escaped form.
    pub fn com(&self) -> &str {
        &self.com
    }

    /// Returns the timestamp of image upload if attachment exists, or None otherwise.
    pub fn tim(&self) -> Option<i64> {
        self.tim
    }

    /// Returns the filename as it appeared on the poster's device if attachment exists, or None otherwise.
    pub fn filename(&self) -> Option<&str> {
        str_opt_ref!(self.filename)
    }

    /// Returns the filetype if attachment exists, or None otherwise.
    pub fn ext(&self) -> Option<&str> {
        str_opt_ref!(self.ext)
    }

    /// Returns the size of uploaded file in bytes if attachment exists, or None otherwise.
    pub fn fsize(&self) -> Option<i32> {
        self.fsize
    }

    /// Returns the MD5 hash of file if attachment exists, or None otherwise.
    pub fn md5(&self) -> Option<&str> {
        str_opt_ref!(self.md5)
    }

    /// Returns the image width if attachment exists, or None otherwise.
    pub fn w(&self) -> Option<i32> {
        self.w
    }

    /// Returns the image height if attachment exists, or None otherwise.
    pub fn h(&self) -> Option<i32> {
        self.h
    }

    /// Returns the thumbnail image width if attachment exists, or None otherwise.
    pub fn tn_w(&self) -> Option<i32> {
        self.tn_w
    }

    /// Returns the thumbnail image height if attachment exists, or None otherwise.
    pub fn tn_h(&self) -> Option<i32> {
        self.tn_h
    }

    /// Returns 1 if file was deleted from post, or None otherwise.
    pub fn filedeleted(&self) -> Option<i32> {
        self.filedeleted
    }

    /// Returns 1 if image was spoilered, or None otherwise.
    pub fn spoiler(&self) -> Option<i32> {
        self.spoiler
    }

    /// Returns the custom spoiler ID for spoilered image, or None otherwise.
    pub fn custom_spoiler(&self) -> Option<i32> {
        self.custom_spoiler
    }

    /// Returns the number of replies minus number of previewed replies.
    pub fn omitted_posts(&self) -> Option<i32> {
        self.omitted_posts
    }

    /// Returns the number of image replies minus number of previewed image replies.
    pub fn omitted_images(&self) -> Option<i32> {
        self.omitted_images
    }

    /// Returns the total number of replies to the thread.
    pub fn replies(&self) -> Option<i32> {
        self.replies
    }

    /// Returns the total number of image replies to the thread.
    pub fn images(&self) -> Option<i32> {
        self.images
    }

    /// Returns true if thread has reached bumplimit, or None otherwise.
    pub fn bumplimit(&self) -> Option<bool> {
        self.bumplimit
    }

    /// Returns true if thread has reached imagelimit, or None otherwise.
    pub fn imagelimit(&self) -> Option<bool> {
        self.imagelimit
    }

    /// Returns the UNIX timestamp of last thread modification.
    pub fn last_modified(&self) -> Option<u64> {
        self.last_modified
    }

    /// Returns the category tag if present, or None otherwise.
    pub fn tag(&self) -> Option<&str> {
        str_opt_ref!(self.tag)
    }

    /// Returns the SEO URL slug for thread.
    pub fn semantic_url(&self) -> Option<&str> {
        str_opt_ref!(self.semantic_url)
    }

    /// Returns the year a 4chan pass was bought if present, or None otherwise.
    pub fn since4pass(&self) -> Option<i32> {
        self.since4pass
    }

    /// Returns the number of unique posters in the thread if present, or None otherwise.
    pub fn unique_ips(&self) -> Option<i32> {
        self.unique_ips
    }

    /// Returns true if a mobile-optimized image exists for the post, or None otherwise.
    pub fn m_img(&self) -> Option<bool> {
        self.m_img
    }

    /// Returns a slice of the most recent replies to the thread if present, or None otherwise.
    pub fn last_replies(&self) -> Option<&[CatPost]> {
        self.last_replies.as_deref()
    }
}
