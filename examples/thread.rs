//! Example demonstrating how to fetch and interact with a specific thread using dot4ch
//!
//! This example shows:
//! - Creating a dot4ch client
//! - Fetching a specific thread from the /po/ board
//! - Updating the thread
//! - Extracting and formatting the original post (OP) comment

use dot4ch::thread::Post;
use dot4ch::thread::Thread;
use dot4ch::Client;

/// Type alias for simplifying error handling
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[tokio::main]
async fn main() -> Result<()> {
    // Create a new dot4ch client
    let client = Client::new();

    // Fetch a specific thread from the /po/ board
    // In this example, we're fetching thread number 570368
    let mut thread = Thread::new(&client, "po", 570368).await?;

    // Update the thread to get the latest posts
    thread.update(&client).await?;

    // Extract the comment from the original post (OP)
    if let Some(comment) = thread.first().and_then(Post::com) {
        // Replace HTML line breaks with actual newlines for readability
        let formatted_comment = comment.replace("<br>", "\n");

        println!("Original Post Comment:");
        println!("{formatted_comment}");
    } else {
        println!("No comment found for the original post.");
    }

    Ok(())
}
