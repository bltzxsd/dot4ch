//! This example shows:
//! - Creating a dot4ch client
//! - Fetching boards
//! - Updating boards
//! - Extracting the board name from the boards

use dot4ch::board::{Board, Boards};
use dot4ch::Client;

/// Type alias for simplifying error handling
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[tokio::main]
async fn main() -> Result<()> {
    // Create a new dot4ch client
    let client = Client::new();

    // Fetch the boards collection
    let boards = Boards::new(&client).await?;

    // you can update boards but they are unlikely to change.
    // boards.update(&client).await?;

    // Extract the name of the alphabetically first board
    match boards.first().map(Board::board) {
        Some(name) => println!("first board: {name}"),
        _ => println!("No comment found in the catalog."),
    }

    Ok(())
}
