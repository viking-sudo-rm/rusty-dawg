# Rusty DAWG

A library for building suffix automata for string indexing and searching in Rust.

For most basic use cases, it might be easier to use the Python wrapper [py-rusty-dawg](https://github.com/viking-sudo-rm/py-rusty-dawg).

## Building

```
cargo test
cargo build --release
```

## Example Usage

After a release build, you can run:

```
./target/release/rusty-dawg \
    --train-path /path/like/train.txt \
    --test-path /path/like/val.txt \
    --save-path /path/like/wikitext2.dawg \
    --results-path /path/like/wikitext2.json \
    --tokenize
```

The last `tokenize` flag specifies that the data is Unicode/ASCII text that should be tokenized by splitting whitespace. If the flag is omitted, it is assumed that the data is a list of integers representing token IDs encoded in ASCII.

## Contributions

Very welcome! There are lots of interesting algorithmic improvements under the hood to make Rusty DAWG more efficient and scalable. Get in contact if you want to help out!