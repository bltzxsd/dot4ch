use dot4ch::{board::Board, Client, Update};
use simple_logger::SimpleLogger;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // setting up logging.
    SimpleLogger::new().init()?;

    // We first make a client:
    let client = Client::new();

    // Then we build our board.
    // I'll pick /g/ as our board.
    // Building a board does take quite a bit of time.
    let g = Board::build(&client, "g").await?;

    // We will get the specific thread from here:
    let bpg = 81730319;

    let thread = &g.get(bpg).expect("No thread by that number was found!");
    println!("{:?}", thread);

    // After a while has passed we can update the new board.
    let _ = g.update().await?;

    Ok(())
}
