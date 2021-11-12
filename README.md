# Transaction Engine

A toy transaction engine for a very specific data format.

## Usage

The library portion of this crate exposes both a single and multi-threaded engine. The main difference being that the internal state is wrapped in an `Arc`/`RwLock` in the thread-safe version. If you don't need to process multiple streams, the single threaded engine should have less overhead.

### Single Threaded CSV

The default binary uses the single threaded engine to parse a csv file input and, when finished, writes the state of all accounts out to a new csv:

```sh
cargo run -- ./transactions.csv > ./accounts.csv
```

## Assumptions

A few additional assumptions are made in the implementation of this library:

- Any transaction against a locked account should fail (i.e. a locked account cannot be disputed)
- We aren't interested in logging what actions are skipped. Error handling in the binary (not the library) is mostly just to ignore actions that cannot be parsed or generate errors (since stdout is taken for output)
- The 4 decimal precision required in the format is a hard requirement (i.e. output values should be rounded to 4 decimal places). Because of this, the `rust_decimal` crate is used. To just use a `f64`'s for all float parsing and display, disable the crate feature `decimal`. The decimal rounding strategy used is `MidpointAwayFromZero` as opposed to the default `BankersRounding`, just because that seems the most familiar to me and honestly never knew there were so many rounding strategies.

## Unresolved Questions and Future Work

Besides the `TODO`'s littered throughout the code, there are a few items that would make great future improvements, mainly expanding modularity, handling more edge cases, adding a logging framework (gotta be `tracing`), and implementing the `async` side of things.

A few questions I'm not sure how to handle are:

- If a dispute is placed on a withdrawl (i.e. the held funds would have to be negative, as they cannot come from the available funds), what happens? This could be solved by representing transactions with a reference to both clients and a directionality. Then deposited funds would always be the funds held.
- If we stored the transaction chronology better (maybe a `Vec` of transaction ids), could we better handle failed actions after a transaction is disputed? On second though, in a real system actions should be ephemeral (you don't want your account retrying a withdrawal you made yesterday because some funds are new cleared).

### Crate Structure

If this were a real project, it would be easier to have the binary as a seperate crate (making this a workspace). Then the `csv` dependency would be unneccessary for the library. Extensions and other engines could be seperated out into crates as well.

### Testing

Normal `cargo`-based unit tests are available, but they could be expanded (68% from `tarpaulin`, but I don't think that covers all the ignored edge cases). Additionally, it would be nice to include some integration tests for csv parsing (possibly just using the built `csv-engine` executable?). Currently the csv-specific is built into the binary, but it is relatively simplistic. Some fuzz tests would probably also be useful for robustness to noisy/weird/malicious input.

### Logging, Persistence, and Traceability

At the very least, adding logging (though that currently conflicts with piping the csv to stdout) would allow for noting when actions are ignored. Of course, the inner state of the engine is basically a database with `accounts` and `transactions` tables, so putting those in an actual database (in-memory or otherwise) would be a relatively simple change if the dataset grows large. It would also allow persistence of the account states. Depending on how logging is implemented, adding an `actions` table could be useful for traceability.

### Bright, Shiny Async

Currently, the beginnings of what might be necessary for a more complex, async implementation exist, but are unfinished (since there are a lot of directions it could go). The library design makes this easier, as you can just add more binaries for implementations. Some possible additions:

- Flesh out the `AsyncEngine` trait to accept both `Stream`s and regular iterators.
- Implement the async engine as a `tower` `Service`, or just spawn a `hyper` server and share the `MultiThreadedEngine` across spawned threads and to incoming data as either individual csv files or iterators over chunks of `Actions`.
- Websockets/gRPC connections streaming `Action`s?

## Contributing

Don't contribute to this, it's just a coding test!

But if you want to, the `rustfmt` config in this repository contains some unstable features. You can format the crate with `cargo +nightly fmt` to avoid warnings.
