use dot4ch::{threadlist::Catalog, Client, Update};
use simple_logger::SimpleLogger;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // setting up logging.
    SimpleLogger::new().init()?;

    // We first make a client:
    let client = Client::new();

    // Then we make a catalog for our desired board.
    // Here I'll pick /g/
    let g = Catalog::new(&client, "g").await?;

    // You can directly print the catalog.
    // For more information on what's printed,
    // I suggest you check out the fmt::Display impl on threadlist::Catalog
    println!("{}", g);

    // Okay, now that some time has passed we want to update the catalog.
    let g = g.update(&client).await?;

    // and print that
    println!("{}", g);

    Ok(())
}
