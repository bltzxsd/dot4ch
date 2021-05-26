# dot4ch

dot4ch is a convenient wrapper library around 4chan's API.

This library can fetch and update:

- Posts
- Threads
- Boards

While respecting 4chan's:

- GET 1 second-per-request cooldown.
- `If-Modified-Since` headers with update requests.
- 10 second cooldown with `Thread`, `Catalog` and `Board` update requests.

## THE DOCS

[CLICK HERE](<https://docs.rs/dot4ch/*/dot4ch/>)

## Example: Getting an image from the OP of a thread

```rust

#[tokio::main]
async fn main() {
    let mut client = Client::new();

    let board = "g";

    let post_id = 81743559;

    let thread = Thread::new(&client, board, post_id).await.unwrap();
    
    let post = thread.op();
    println!("{}", post.image_url(board).unwrap());
}

```
