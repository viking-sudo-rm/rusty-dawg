# Rusty DAWG

A library for building suffix automata for string indexing and searching in Rust.

For Python bindings of Rusty DAWG (easier for basic use cases), see: [py-rusty-dawg](https://github.com/viking-sudo-rm/py-rusty-dawg)

## Building

```
cargo test
cargo build --release
```

## Local Usage

```
./target/debug/rusty-dawg /Users/willm/Desktop/wikitext-2-raw/wiki.train.raw /Users/willm/Desktop/wikitext-2-raw/wiki.valid.raw /Users/willm/Desktop/wikitext2.json
```

## GCP Usage

```
sudo /home/willm/miniconda3/envs/dawg/bin/python scripts/get_corpus.py
```