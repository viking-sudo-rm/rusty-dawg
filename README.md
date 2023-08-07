# Rusty DAWG

A library for building suffix automata for string indexing and searching in Rust.

For most basic use cases, it might be easier to use the Python wrapper [py-rusty-dawg](https://github.com/viking-sudo-rm/py-rusty-dawg).

# How to Use

## Building

```
cargo test
cargo build --release
```

## Running Benchmarking Script

To run the benchmarking script, you first need to download the [data](https://drive.google.com/file/d/1XRZA2eki_Z8M0QrYN4BrbN7dghMYqYby/view?usp=sharing) directory, unzip it, and put it in the root of the repository directory (i.e., rusty-dawg/data). If you'd prefer, you can also retrieve the data directories for Wikitext2 and Wikitext103 from /home/willm/splits on NFS.

Now you will be able to benchmark building the DAWG on Wikitext2 or Wikitext103!

```
source scripts/benchmark.sh wikitext-2-raw
source scripts/benchmark.sh wikitext-103-raw
```

The total runtime will be printed out by the script's progress bar. You can find the size of the resulting DAWG by looking at:

```
ls -l /tmp/wikitext-2-raw.dawg
ls -l /tmp/wikitext-103-raw.dawg
```

## Contributions

Very welcome! There are lots of interesting algorithmic improvements under the hood to make Rusty DAWG more efficient and scalable. Get in contact if you want to help out!