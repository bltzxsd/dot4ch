# Examples

Some basic examples to get you started.

Do keep in mind these will not work in the future and are just references as 4chan threads are quite fast
and keep getting deleted/archived etc.

## Logging

Logging is handled by the [Log Crate](<https://github.com/rust-lang/log>)
and for posting the logs, I use [Simple Logger](<https://github.com/borntyping/rust-simple_logger>).

But anything else you may want to use is fine too.

While running the examples be sure to set the environment variable `RUST_LOG` to something, such as `INFO` or `DEBUG` to see the logs.

## Dependencies

Check out the `Cargo.toml` above.

The logging crates are optional.
The `tokio` crate only requires this feature in a binary/build:

- `rt-multi-thread`
- `macros`

## Questions

If you have any questions that you would like to ask,
feel free to open a PR.
