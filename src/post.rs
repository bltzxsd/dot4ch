//! Holds an information of a single post
//!
//! Some of the fields here are optional
//!
//! Posts are usually used in `Thread`s which is why they do not have a `new()` but they do have an `Default` implementation.
//!
//! ## 4chan API:
//! /[board]/thread/[op ID].json files are a representation of a single OP and all the replies, which form a thread.
//!
//! ```rust
//! use dot4cha::post::Post;
//!
//! let z = Post::default();
//!
//! println!("{}", z.id());
//!
//! assert_eq!(z.id(), 0);
//! ```

use std::fmt::{Display, Formatter};
use crate::default;
use serde::{Deserialize, Serialize};

/// The Post represents a derserialized post from a thread.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Post {
    /// The numeric post ID
    no: u32,

    /// For replies: this is the ID of the thread being replied to.
    ///
    /// For OP: this value is zero
    resto: u32,

    /// If the thread is being pinned to the top of the page
    #[serde(default = "default::<u8>")]
    sticky: u8,

    /// If the thread is closed to replies
    #[serde(default = "default::<u8>")]
    closed: u8,

    /// MM/DD/YY(Day)HH:MM (:SS on some boards), EST/EDT timezone
    now: String,

    /// UNIX timestamp the post was created
    time: i64,

    /// Name user posted with. Defaults to `Anonymous`
    #[serde(default = "default::<String>")]
    name: String,

    /// The user's tripcode
    #[serde(default = "default::<String>")]
    trip: String,

    /// The poster's ID
    #[serde(default = "default::<String>")]
    id: String,

    /// The capcode identifier for a post
    #[serde(default = "default::<String>")]
    capcode: String,

    /// Poster's ISO 3166-1 alpha-2 country code
    #[serde(default = "default::<String>")]
    country: String,

    /// Poster's country name
    #[serde(default = "default::<String>")]
    country_name: String,

    /// Poster's board flag code
    #[serde(default = "default::<String>")]
    board_flag: String,

    /// Poster's board flag name
    #[serde(default = "default::<String>")]
    flag_name: String,

    /// OP Subject text
    #[serde(default = "default::<String>")]
    sub: String,

    /// Comment (HTML escaped)
    #[serde(default = "default::<String>")]
    com: String,

    /// Unix timestamp + microtime that an image was uploaded
    #[serde(default = "default::<u64>")]
    tim: u64,

    /// Filename as it appeared on the poster's device
    #[serde(default = "default::<String>")]
    filename: String,

    /// Filetype
    #[serde(default = "default::<String>")]
    ext: String,

    /// Size of uploaded file in bytes
    #[serde(default = "default::<u32>")]
    fsize: u32,

    /// 24 character, packed base64 MD5 hash of file
    #[serde(default = "default::<String>")]
    md5: String,

    /// Image Width Dimension
    #[serde(default = "default::<u32>")]
    w: u32,

    /// Image Height Dimension
    #[serde(default = "default::<u32>")]
    h: u32,

    /// Thumbnail image width dimension
    #[serde(default = "default::<u32>")]
    tn_w: u32,

    /// Thumbnail image height dimension
    #[serde(default = "default::<u32>")]
    tn_h: u32,

    /// If the file was deleted from the post
    #[serde(default = "default::<u8>")]
    filedeleted: u8,

    /// If the image was spoiler'd or not
    #[serde(default = "default::<u8>")]
    spoiler: u8,

    /// The custom spoiler ID for a spoilered image
    #[serde(default = "default::<u8>")]
    custom_spoiler: u8,

    /// Total number of replies to a thread
    #[serde(default = "default::<u32>")]
    replies: u32,

    /// Total number of image replies to a thread
    #[serde(default = "default::<u32>")]
    images: u32,

    /// If a thread has reached bumplimit, it will no longer bump
    #[serde(default = "default::<u8>")]
    bumplimit: u8,

    /// If an image has reached image limit, no more image replies can be made
    #[serde(default = "default::<u8>")]
    imagelimit: u8,

    /// The category of .swf upload
    #[serde(default = "default::<String>")]
    tag: String,

    /// SEO URL slug for thread
    #[serde(default = "default::<String>")]
    semantic_url: String,

    /// Year 4chan pass bought
    #[serde(default = "default::<u16>")]
    since4pass: u16,

    /// Number of unique posters in a thread
    #[serde(default = "default::<u16>")]
    unique_ips: u16,

    /// Mobile optimized image exists for post
    #[serde(default = "default::<u8>")]
    m_img: u8,

    /// Thread has reached the board's archive  
    #[serde(default = "default::<u8>")]
    archived: u8,

    /// UNIX timestamp the post was archived
    #[serde(default = "default::<i64>")]
    archived_on: i64,
}

impl Post {
    /// Gets the post number of a Post
    pub fn id(&self) -> u32 {
        self.no
    }

    /// Get the subject from the text
    pub fn subject(&self) -> &str {
        &self.sub
    }

    /// Gets the time from Post
    ///
    /// format:
    /// MM/DD/YY(Day)HH:MM (:SS on some boards), EST/EDT timezone
    pub fn time_now(&self) -> &str {
        &self.now
    }

    /// Gets the comment from the Post
    pub fn content(&self) -> &str {
        &self.com
    }

    /// Gets the filename
    pub fn filename(&self) -> &str {
        &self.filename
    }

    /// Gets the filename's extension
    pub fn ext(&self) -> &str {
        &self.ext
    }

    /// Gets the number of replies to the Post
    pub fn replies(&self) -> u32 {
        self.replies
    }

    /// Check if the OP Post is archived.
    pub fn archived(&self) -> bool {
        if self.archived == 1 {
            return true;
        }
        false
    }

    /// Returns the dimensions of an image in a tuple: (WIDTH, HEIGHT)
    pub fn image_dimensions(&self) -> (u32, u32) {
        (self.w, self.h)
    }

    /// Returns the UNIX timestamp of when the post was archived
    pub fn archived_on(&self) -> i64 {
        self.archived_on
    }

    /// Returns the 4chan image url from the supplied post.
    pub fn image_url(&self, board: &str) -> Option<String> {
        if !self.filename.is_empty() && !self.ext.is_empty() {
            Some(format!(
                "https://i.4cdn.org/{}/{}{}",
                board,
                &self.tim,
                &self.ext
            ))
        } else {
            None
        }
    }
}

impl Display for Post {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let fmt = format!(
            "Post ID: {}\n Subject: {}\n Content: {}\n",
            self.id, self.sub, self.com
        );
        write!(f, "{}", fmt)
    }
}
