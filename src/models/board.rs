use std::collections::HashMap;

use crate::{
    client::Reply,
    error::Error::{self, MissingHeader},
    models::{de_bool, maybe_de_bool, Metadata},
    result::Result,
    Client,
};
use reqwest::header::LAST_MODIFIED;
use serde::{Deserialize, Serialize};

/// A collection representing a list of boards and their detailed attributes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Boards {
    boards: Vec<Board>,
    #[serde(skip)]
    pub(crate) metadata: Metadata,
}

impl Boards {
    /// Constructs a new `Boards` collection by fetching the list of all boards.
    ///
    /// # Errors
    ///
    /// Returns an error if the fetch operation fails or if required headers are lacking.
    pub async fn new(client: &Client) -> Result<Self> {
        let url = String::from("https://a.4cdn.org/boards.json");
        let reply: Reply<Boards> = client.fetch_json(&url, None).await?;
        let last_modified = reply
            .last_modified
            .ok_or_else(|| MissingHeader(LAST_MODIFIED))?;

        let mut boards = reply.inner?;
        let metadata = Metadata { url, last_modified };
        boards.metadata = metadata;

        Ok(boards)
    }

    /// Updates the current `Boards` collection with fresh data.
    ///
    /// Overwrites existing board information with the latest fetched data.
    ///
    /// # Errors
    ///
    /// Returns an error if updating fails due to fetch issues.
    pub async fn update(&mut self, client: &Client) -> Result<()> {
        let reply: Reply<Self> = client
            .fetch_json(self.metadata.url(), Some(&self.metadata.last_modified))
            .await?;

        match reply.inner {
            Ok(i) => *self = i,
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

impl std::ops::Deref for Boards {
    type Target = Vec<Board>;

    fn deref(&self) -> &Self::Target {
        &self.boards
    }
}

/// Represents an individual board with its attributes.
///
/// Provides detailed information such as the number of pages, file size limits,
/// permissions, and optional features for postings.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Board {
    /// The directory the board is located in.
    board: String,

    /// The readable title at the top of the board.
    title: String,

    /// True if the board is worksafe.
    #[serde(deserialize_with = "de_bool")]
    ws_board: bool,

    /// Number of threads on a single index page.
    per_page: u32,

    /// Number of index pages per board.
    pages: u32,

    /// Maximum file size allowed for non `.webm` attachments in KB.
    max_filesize: u32,

    /// Maximum file size allowed for `.webm` attachments in KB.
    max_webm_filesize: u32,

    /// Maximum number of characters allowed in post comment.
    max_comment_chars: u32,

    /// Maximum duration of a `.webm` attachment in seconds.
    max_webm_duration: u32,

    /// Maximum number of replies allowed before a thread stops bumping.
    bump_limit: u32,

    /// Maximum number of image replies per thread before image replies are discarded.
    image_limit: u32,

    /// See [`Cooldowns`]
    cooldowns: Cooldowns,

    /// SEO meta description content for a board.
    meta_description: String,

    /// True if spoilers are enabled.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "maybe_de_bool"
    )]
    spoilers: Option<bool>,

    /// Number of custom spoilers a board has.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    custom_spoilers: Option<u32>,

    /// True if archives are enabled for the board.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "maybe_de_bool"
    )]
    is_archived: Option<bool>,

    /// [`HashMap`] of flag codes mapped to flag names.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    board_flags: Option<HashMap<String, String>>,

    /// True if flags showing poster's country are enabeld.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "maybe_de_bool"
    )]
    country_flags: Option<bool>,

    /// True if poster IDs are enabled on the board.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "maybe_de_bool"
    )]
    user_ids: Option<bool>,

    /// True if users can submit drawings via Oekaki app.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "maybe_de_bool"
    )]
    oekaki: Option<bool>,

    /// True if users can submit SJIS drawings using `[sjis]` tag.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "maybe_de_bool"
    )]
    sjis_tags: Option<bool>,

    /// True if board supports code syntax highlighting using `[code]` tags.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "maybe_de_bool"
    )]
    code_tags: Option<bool>,

    /// True if board supports `[math]` TeX and `[eqn]` tags.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "maybe_de_bool"
    )]
    math_tags: Option<bool>,

    /// True if image posting is disabled on the board.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "maybe_de_bool"
    )]
    text_only: Option<bool>,

    /// True if the name field is disabled on the board.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "maybe_de_bool"
    )]
    forced_anon: Option<bool>,

    /// True if `.webm` attachments with audio are allowed.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "maybe_de_bool"
    )]
    webm_audio: Option<bool>,

    /// True if OPs require a subject.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "maybe_de_bool"
    )]
    require_subject: Option<bool>,

    /// The minimum supported width for an image in pixels.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    min_image_width: Option<u32>,

    /// The maximum supported height of an image in pixels.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    min_image_height: Option<u32>,
}

impl PartialEq for Board {
    fn eq(&self, other: &Self) -> bool {
        self.board == other.board
    }
}

/// Represents the cooldown periods for creating threads, posting replies,
/// and uploading images on a board.
///
/// # Warning
///
/// This field was left undocumented. This struct does not hold and guarantees.
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct Cooldowns {
    /// Cooldown time (in seconds) before a user can create another thread.
    threads: u32,
    /// Cooldown time (in seconds) before a user can post another reply.
    replies: u32,
    /// Cooldown time (in seconds) before a user can upload another image.
    images: u32,
}

impl Cooldowns {
    /// Returns cooldown time (in seconds) before a user can upload another image.
    pub fn images(&self) -> u32 {
        self.images
    }

    /// Returns cooldown time (in seconds) before a user can post another reply.
    pub fn replies(&self) -> u32 {
        self.replies
    }

    /// Returns cooldown time (in seconds) before a user can create another thread.
    pub fn threads(&self) -> u32 {
        self.threads
    }
}

impl Board {
    /// Returns the directory the board is located in.
    pub fn board(&self) -> &str {
        &self.board
    }

    /// Returns the readable title at the top of the board.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns true if the board is worksafe.
    pub fn ws_board(&self) -> bool {
        self.ws_board
    }

    /// Returns how many threads are on a single index page.
    pub fn per_page(&self) -> u32 {
        self.per_page
    }

    /// Returns number of index pages the board has.
    pub fn pages(&self) -> u32 {
        self.pages
    }

    /// Returns maximum file size allowed for non `.webm` attachments in KB.
    pub fn max_filesize(&self) -> u32 {
        self.max_filesize
    }

    /// Returns maximum file size allowed for `.webm` attachments in KB.
    pub fn max_webm_filesize(&self) -> u32 {
        self.max_webm_filesize
    }

    /// Returns maximum number of characters allowed in a post comment.
    pub fn max_comment_chars(&self) -> u32 {
        self.max_comment_chars
    }

    /// Returns maximum duration of a `.webm` attachment (in seconds).
    pub fn max_webm_duration(&self) -> u32 {
        self.max_webm_duration
    }

    /// Returns maximum number of replies allowed to a thread before the thread stops bumping.
    pub fn bump_limit(&self) -> u32 {
        self.bump_limit
    }

    /// Returns maximum number of image replies per thread before image replies are discarded.
    pub fn image_limit(&self) -> u32 {
        self.image_limit
    }

    /// This is left undocumented in
    pub fn cooldowns(&self) -> &Cooldowns {
        &self.cooldowns
    }

    ///Returns SEO meta description content for a board.
    pub fn meta_description(&self) -> &str {
        &self.meta_description
    }

    /// Returns true if spoilers enabled on the board.
    pub fn spoilers(&self) -> Option<bool> {
        self.spoilers
    }

    /// Returns the number of custom spoilers the board has.
    pub fn custom_spoilers(&self) -> Option<u32> {
        self.custom_spoilers
    }

    /// Returns true if archives are enabled for the board.
    pub fn is_archived(&self) -> Option<bool> {
        self.is_archived
    }

    /// Returns an array of flag codes mapped to flag names.
    pub fn board_flags(&self) -> Option<&HashMap<String, String>> {
        self.board_flags.as_ref()
    }

    /// Returns true if flags showing the poster's country are enabled on the board.
    pub fn country_flags(&self) -> Option<bool> {
        self.country_flags
    }

    /// Returns true if the poster ID tags are enabled on the board.
    pub fn user_ids(&self) -> Option<bool> {
        self.user_ids
    }

    /// Returns true if users can submit drawings via the Oekaki app.
    pub fn oekaki(&self) -> Option<bool> {
        self.oekaki
    }

    /// Returns true if users submit sjis drawings using the `[sjis]` tags.
    pub fn sjis_tags(&self) -> Option<bool> {
        self.sjis_tags
    }

    /// Retursn true if board supports code syntax highlighting using the `[code]` tags.
    pub fn code_tags(&self) -> Option<bool> {
        self.code_tags
    }

    /// Returns true if board supports `[math]` TeX and `[eqn]` tags.
    pub fn math_tags(&self) -> Option<bool> {
        self.math_tags
    }

    /// Returns true if image posting is diabled for the board.
    pub fn text_only(&self) -> Option<bool> {
        self.text_only
    }

    /// Returns true if the name field is disabled on the board/
    pub fn forced_anon(&self) -> Option<bool> {
        self.forced_anon
    }

    /// Returns true if webms with audio allowed on the board.
    pub fn webm_audio(&self) -> Option<bool> {
        self.webm_audio
    }

    /// Returns true if OPs require a subject.
    pub fn require_subject(&self) -> Option<bool> {
        self.require_subject
    }

    /// Returns the minimum image width (in pixels).
    pub fn min_image_width(&self) -> Option<u32> {
        self.min_image_width
    }

    /// Returns the minimum image height (in pixels).
    pub fn min_image_height(&self) -> Option<u32> {
        self.min_image_height
    }
}
