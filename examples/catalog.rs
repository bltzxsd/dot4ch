//! Example demonstrating how to fetch and interact with a board's catalog using dot4ch
//!
//! This example shows:
//! - Creating a dot4ch client
//! - Fetching the catalog for a board
//! - Updating the catalog
//! - Extracting the comment from the first thread on the first catalog page

use dot4ch::catalog::Catalog;
use dot4ch::Client;

/// Type alias for simplifying error handling
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[tokio::main]
async fn main() -> Result<()> {
    // Create a new dot4ch client
    let client = Client::new();

    // Fetch the catalog for the /po/ (Papercraft) board
    let mut catalog = Catalog::new(&client, "po").await?;

    // Update the catalog to ensure we have the latest information
    catalog.update(&client).await?;

    // Extract the comment from the first thread on the first catalog page
    if let Some(comment) = catalog
        .first()
        .and_then(|page| page.threads().first())
        .map(|thread| thread.com())
    {
        // Replace HTML line breaks with actual newlines for readability
        let formatted_comment = comment.replace("<br>", "\n");

        println!("First Thread on Catalog Comment:");
        println!("{formatted_comment}");
    } else {
        println!("No comment found in the catalog.");
    }

    Ok(())
}
