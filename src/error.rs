use reqwest::{header::HeaderName, StatusCode};
use std::error::Error as StdError;
use std::fmt;
use tokio::sync::AcquireError;

#[derive(Debug)]
pub enum Error {
    /// Reqwest Errors
    Http(reqwest::Error),
    /// unexpected reponse status code
    UnexpectedStatus(StatusCode),
    /// Missing required header in response
    MissingHeader(HeaderName),
    /// rate limiting error
    RateLimit(AcquireError),
    /// nothing to update
    NotModified,
}

// Implement `std::fmt::Display` for pretty-printing the error messages
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Http(err) => write!(f, "failed to parse response: {err}"),
            Error::UnexpectedStatus(code) => write!(f, "unexpected status: {code}"),
            Error::MissingHeader(header) => write!(f, "missing header: {header}"),
            Error::RateLimit(err) => write!(f, "rate limit error: {err}"),
            Error::NotModified => write!(f, "not modified"),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::Http(err) => Some(err),
            Error::RateLimit(err) => Some(err),
            _ => None,
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::Http(err)
    }
}

impl From<AcquireError> for Error {
    fn from(err: AcquireError) -> Self {
        Error::RateLimit(err)
    }
}
