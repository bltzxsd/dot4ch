//! This example shows:
//! - Creating a dot4ch client
//! - Fetching the thread list for a board
//! - Updating the thread list
//! - Extracting the thread number of the first thread

use dot4ch::threadlist::ThreadList;
use dot4ch::Client;

/// Type alias for simplifying error handling
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[tokio::main]
async fn main() -> Result<()> {
    // Create a new dot4ch client
    let client = Client::new();

    // Fetch the thread list for the /po/ (Papercraft) board
    let mut threadlist = ThreadList::new(&client, "po").await?;

    // Update the thread list so we get newest threads.
    threadlist.update(&client).await?;

    // Extract the thread number from the first thread
    match threadlist
        .first()
        .and_then(|x| x.threads().first())
        .map(|thread| thread.no())
    {
        Some(thread_no) => println!("First Thread Number: {thread_no}"),
        _ => println!("No threads found in the list."),
    }

    Ok(())
}
