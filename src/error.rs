use thiserror::Error;

#[derive(Debug, Error)]
pub enum DotError {
    #[error("{0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("could not start up the client")]
    ClientFormation,

    #[error("{0}")]
    Update(String),

    #[error("{0}")]
    Thread(String),

    #[error("thread {} is archived at {}", .thread, .time)]
    Archived { time: String, thread: u32 },

    #[error("{}", _0)]
    Chrono(#[from] chrono::ParseError),

    #[error("{}", _0)]
    IO(#[from] std::io::Error),
}