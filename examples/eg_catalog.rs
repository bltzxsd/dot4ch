use dot4ch::{Catalog, Client, Update};
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
    println!("{:?}", g);

    // You can also get a slice of Pages in from the Catalog.
    println!("{:#?}", &g[..5]);

    // get the the first page
    let page = g[0].to_owned();

    // we can also convert all CatalogThreads to actual threads.
    for cat_thread in page.threads() {
        let common_thread = cat_thread.to_thread(&client, "g").await?;
        println!("{:?}", common_thread);
    }

    // Okay, now that some time has passed we want to update the catalog.
    let g = g.update().await?;

    // and print that
    println!("{:?}", &g);

    Ok(())
}
