//! Holds an information of a single post
//!
//! Some of the fields here are optional
//!
//! Posts are usually used in [`crate::thread::Thread`]s which is why they do not have a `new()` but they do have an [`Default`] implementation.
//!
//! ## 4chan API:
//! /<board>/<thread>/<op ID>.json files are a representation of a single OP and all the replies, which form a thread.
//!
//! ```
//! use dot4ch::post::Post;
//!
//! let z = Post::default();
//!
//! println!("{}", z.id());
//!
//! assert_eq!(z.id(), 0);
//! ```

use crate::default;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

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
    /// Returns the post number of a Post
    pub fn id(&self) -> u32 {
        self.no
    }

    /// Returns the subject from the text.
    ///
    /// Returns an empty str if there isnt any.
    pub fn subject(&self) -> &str {
        &self.sub
    }

    /// Returns the time from Post
    ///
    /// format:
    /// MM/DD/YY(Day)HH:MM (:SS on some boards), EST/EDT timezone
    pub fn time_now(&self) -> &str {
        &self.now
    }

    /// Returns the comment from the Post
    pub fn content(&self) -> &str {
        &self.com
    }

    /// Returns the filename if there is one or an empty string otherwise.
    pub fn filename(&self) -> &str {
        &self.filename
    }

    /// Returns the filename's extension if there is a file.
    ///
    /// Returns an empty &str otherwise.
    pub fn ext(&self) -> &str {
        &self.ext
    }

    /// Returns the number of replies to the Post
    pub fn replies(&self) -> u32 {
        self.replies
    }

    /// Returns true if the post is archived. False othwrwise.
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
        if self.filename.is_empty() {
            None
        } else {
            Some(format!(
                "https://i.4cdn.org/{}/{}{}",
                board, &self.tim, &self.ext
            ))
        }
    }

    /// Returns a UNIX Timestamp of when the post was created
    pub fn post_time(&self) -> i64 {
        self.time
    }

    /// Returns a true if the thread is pinned
    pub fn sticky(&self) -> bool {
        if self.sticky != 0 {
            return true;
        }
        false
    }

    /// Returns true if the thread is closed to replies
    pub fn closed(&self) -> bool {
        if self.closed != 0 {
            return true;
        }
        false
    }

    /// Returns the tripcode if the poster has one. Returns `None` otherwise.
    pub fn tripcode(&self) -> Option<&str> {
        if self.trip.is_empty() {
            return None;
        }
        Some(&self.trip)
    }

    /// Returns the capcode identifier for a post if there is one. `None` otherwise.
    pub fn capcode(&self) -> Option<&str> {
        if self.capcode.is_empty() {
            return None;
        }
        Some(&self.capcode)
    }

    /// Returns the poster's country name if there is one avaliable. `None` otherwise.
    pub fn country(&self) -> Option<&str> {
        if self.country_name.is_empty() {
            return None;
        }
        Some(&self.country_name)
    }

    /// Returns the post's file's MD5 hash if there is one.
    pub fn md5hash(&self) -> Option<&str> {
        if self.md5.is_empty() {
            return None;
        }
        Some(&self.md5)
    }

    /// Returns true if the file in the post was deleted.
    pub fn file_deleted(&self) -> bool {
        if self.filedeleted != 0 {
            return true;
        }
        false
    }

    /// Returns a filesize of a post if it has one.
    pub fn filesize(&self) -> Option<u32> {
        if self.fsize != 0 {
            return Some(self.fsize);
        }
        None
    }

    /// Returns true if the thread has reached image limit, false otherwise
    pub fn image_limit(&self) -> bool {
        if self.imagelimit != 0 {
            return true;
        }
        false
    }

    /// Returns true if the thread has reached bump limit, false otherwise
    pub fn bump_limit(&self) -> bool {
        if self.bumplimit != 0 {
            return true;
        }
        false
    }
}

impl Display for Post {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let status = format!("Archived: {} | Closed: {}", self.archived(), self.closed());
        let fmt = format!(
            "Post ID: {}, Status: {}\n Subject: {}\n Content: {}\n",
            self.no, &status, self.sub, self.com
        );
        write!(f, "{}", fmt)
    }
}
