//! This example shows:
//! - Creating a dot4ch client
//! - Fetching the archive for a board
//! - Updating the archive
//! - Extracting and printing the oldest archived thread number

use dot4ch::archive::Archive;
use dot4ch::Client;

/// Type alias for simplifying error handling
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[tokio::main]
async fn main() -> Result<()> {
    // Create a new dot4ch client
    let client = Client::new();

    // Fetch the archive for the /po/ (Papercraft) board
    let mut archive = Archive::new(&client, "po").await?;

    // Update the archive to fetch newly archived threads.
    archive.update(&client).await?;

    // Extract the oldest archived thread number
    match archive.last() {
        Some(no) => println!("Oldest Archived Thread: {no}"),
        _ => println!("No archived threads found."),
    }

    Ok(())
}
