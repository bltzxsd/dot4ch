# dot4ch

[![Rust](https://github.com/bltzxsd/dot4ch/actions/workflows/rust.yml/badge.svg)](https://github.com/bltzxsd/dot4ch/actions/workflows/rust.yml)

dot4ch is a convenient wrapper library around 4chan's API.

This library can fetch and update:

- Posts
- Threads
- Boards

While respecting 4chan's:

- GET 1 second-per-request cooldown.
- `If-Modified-Since` headers with update requests.
- 10 second cooldown with `Thread`, `Catalog` and `Board` update requests.

## Getting Started

**[THE DOCS](<https://docs.rs/dot4ch/*/dot4ch/>)**

There are plenlty of examples in the [examples directory](<https://github.com/bltzxsd/dot4ch/tree/master/examples>) to get you
started plus the I believe everything in the crate is documented.

You can run any example with

```shell
cargo run --example <example name>
```
