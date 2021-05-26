use simple_logger::SimpleLogger;
use dot4ch::{Client, Update, thread::Thread};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // setting up the logger
    SimpleLogger::new().init()?;

    // We make a client
    let client = Client::new();
    
    // some let bindings that we will use
    let board = "g";
    let post: u32 = 81730319; // This is the /g/bpg/ or The Beginner Programmer's General #29. 
    // Do keep in mind that this example might not work in the future 
    // since this thread will be archived/404'd after a certain time due to various factors

    // Lets make a new thread from the data we have.
    let sample_thread = Thread::new(&client, board, post).await?;

    // We can find any post in this thread by its id
    println!("{}", sample_thread.find(81730461u32)?);

    // Get the thread URL 
    println!("{}", sample_thread.thread_url());

    // Get the OP 
    println!("{}", sample_thread.op());

    // Say we want to update the thread 
    let _ = sample_thread.update(&client).await?;
    // This will either return a 304 Not Modified with our thread or a 200 OK with an updated thread.

    Ok(())

}
