# dot4ch

dot4ch is a convenient wrapper library around an imageboard's API.

This library can fetch and update:

- Threads
- Boards
- Catalogs

While respecting:

- GET 1 second-per-request cooldown.
- `If-Modified-Since` headers with update requests.
- 10 seconds per thread update rate limits.

## Getting Started

Examples can be found in the examples directory.

You can run any example with

```text
cargo run --example <example name>
```

## MSRV

Rust MSRV: 1.70.1
Edition: 2021
