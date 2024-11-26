# dot4ch Library Examples

This directory contains example implementations demonstrating various features of the `dot4ch` library for interacting with 4chan.

## Prerequisites

- Rust (latest stable version)

## Examples

### 1. Thread Example (`thread.rs`)

- Creates a client
- Fetches a specific thread by board and thread number
- Updates the thread
- Prints the original post (OP) comment

### 2. Catalog Example (`catalog.rs`)

- Creates a client
- Fetches the catalog for the "/po/" board
- Updates the catalog
- Prints the comment from the first thread on the first page

### 3. Thread List Example (`threadlist.rs`)

- Creates a client
- Fetches the thread list for the "/po/" board
- Updates the thread list
- Prints the thread number of the first thread

### 4. Archive Example (`archive.rs`)

- Creates a client
- Fetches the archive for the "/po/" board
- Updates the archive
- Prints the oldest archived thread

## Running Examples

To run an example, use:

```bash
cargo run --example <example_name>
```

For instance:
```bash
cargo run --example thread
```

## Notes

- These examples use the "/po/" (Papercraft) board as an example
- Modify the board and thread numbers as needed for your use case
- Ensure you have an active internet connection
