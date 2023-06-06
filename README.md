# Rusty DAWG

A library for building suffix automata for string indexing and searching in Rust.

```
cargo test
cargo build
```

To build the DAG on the Brown corpus:

```
./target/debug/rusty-dawg /Users/willm/Desktop/wikitext-2-raw/wiki.train.raw /Users/willm/Desktop/wikitext-2-raw/wiki.valid.raw /Users/willm/Desktop/wikitext2.json
```