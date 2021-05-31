use dot4ch::post::Post;
use simple_logger::SimpleLogger;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    SimpleLogger::new().init()?;

    // We will just make a default post for now with everything empty,
    // You can use any post you want from a thread you have in your actual program though.
    let post = Post::default();

    // Lets print the post.
    println!("{}", &post);

    // lets say we only want to get the post's ID
    let id = post.id();
    println!("Post ID = {} <- This should print 0.", id);
    // or subject which only works on the OP and will return an empty &str otherwise.
    let subject = post.subject();
    println!(
        "Post Subject = {} <- This should not print anything.",
        subject
    );

    // lets get the image url too.
    // You will notice that we have to send the corrent board, since posts do not store the which board they are on which is needed to complete the urls.
    // I recommend you store the board you are accessing as a variable.
    let image_url = post.image_url("g");
                    //   ^^^^^^^^^^^^^^
    // Not all posts have images, and our default implementation of a post certainly doesn't. So this will result an None.
    assert!(image_url.is_none());

    // Posts do not have an Update method since they are individual instances and thus do not implement the Update trait.

    Ok(())
}
