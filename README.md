# Rusty DAWG

A library for building suffix automata for string indexing and searching in Rust.

The key features are:
1. Built a DAWG on a corpus with a one-liner. The DAWG can be saved in two formats: a graph stored in RAM and a graph stored on disk.
2. Use Python bindings to load a saved DAWG for doing fast n-gram search (you can also load it in Rust, but we recommend working with the Python API).
3. A web demo for visualizing n-gram search results over a pre-built DAWG.

### Authors

This library was started by Will Merrill and Yanai Elazar as part of an internship project at AI2. Ananya Jha, Rodney Kinney, David Wadden, Pete Walsh have all since contributed core engineering features. We've also appreciated the support of Michal Guerquin, Johann Dahm, and other members of the Beaker team at AI2 for getting the library to run at very large scale.

# Getting Started

## Installing Rust

Simply use the one-liner [here](https://www.rust-lang.org/tools/install).

## Testing and Building Rusty DAWG

To run tests for the repo, you can use Cargo, which should have been installed along with Rust:

```
cargo test
```

To compile an optimized release build, you can run:

```
cargo build --release
```

Note that the `--release` flag is very important for performance. The code will be 10-100x slower without it.

## Building the Python bindings

The Python bindings are generated using [maturin](https://github.com/PyO3/maturin). First install maturin in your Python environment:

```
pip install maturin
```

Then `cd` into the Python bindings directory (`bindings/python`) and run:

```
make install
```

If, for some reason, you cannot run make files, you can alternatively run the following in two steps:

```
python -m maturin build --release
pip install target/wheels/*.whl
```

## Running Benchmarking Script

To run the benchmarking script, you need the Wikitext2/103 data. You can either download this to rusty-dawg/data path or point to an existing repository (easy on beaker, you can use my copy of the data).

### Using a Custom Data Path

You can point to a custom path where the Wikitext data lives. For example, if you're running on Beaker, you can do:

```
DATA=/home/willm/splits ./scripts/benchmark.sh wikitext-2-raw
DATA/home/willm/splits ./scripts/benchmark.sh wikitext-103-raw
```

### (Optional) Downloading the Data Locally

You first need to download the [data](https://drive.google.com/file/d/1XRZA2eki_Z8M0QrYN4BrbN7dghMYqYby/view?usp=sharing) directory, unzip it, and put it in the root of the repository directory (i.e., rusty-dawg/data). Then you can run:

```
./scripts/benchmark.sh wikitext-2-raw
./scripts/benchmark.sh wikitext-103-raw
```

### Interpreting the Output

The benchmarking spreadsheet requests both the runtime and the memory overhead. The total runtime will be printed out by the script's progress bar. The benchmarking script will also print out the size of the DAWG at the bottom.

# Documentation

This library implements the construction of a suffix automaton (or Directed Acyclic Word Graph, i.e., DAWG) on a large corpus. The suffix automaton is a finite-state machine (really, a graph) that can be used for very fast substring matching over the corpus.

The most relevant modules are the following:

### Graph Implementation: `src/graph`

Implements graph types for representing the DAWG. The only one currently supported is `AvlGraph`: a memory-efficient representation of a graph by a single list of nodes and a single list of edges. The list of edges associated with a node are stored by a balanced binary tree. This follows a similar API to the petgraph `Graph` class, but transitions are much more efficient with a large branching factor (`O(b)` -> `O(log b)`).

### DAWG Construction Algorithm: `src/dawg`

Builds a DAWG (represented as a graph) following the classical construction algorithm due to [Blumer (1984)](https://drive.google.com/file/d/1_FjsV3iSo1rA18DLzVpo_w2Zv4OhBWOl/view?usp=sharing).

### Weights of DAWG States: `src/weight`

Roughly speaking, the states/nodes of the DAWG represent different substrings (actually sets of "equivalent" substrings/indices) in the corpus. We associate additional information with each state via a **weight** to record information about that state, such as the length of the substring it represents and optionally its frequency in the corpus.

This is implemented by `src/weight`. There are various options here depending on how much memory we want to use and which quantities we want the weights to track.

### Tokenization: ``src/tokenize``

This module implements two tokenization strategies: either tokenize by whitespace and identify each token string to an integer ID (`TokenIndex`) or assume the data has already been tokenized and is space-separated (`NullTokenIndex`).

By default, main.rs will use `NullTokenIndex`, but you can pass `--tokenize` to use a custom `TokenIndex`.

### Other Modules

Much of the other modules are for n-gram language modeling and can be ignored for our purposes. `src/lms` implements various types of n-gram language models on top of a DAWG. `src/evaluator` implements logic to evaluate these n-gram language models on a test set. `src/stat_utils` implements a library of basic statistical functions for n-gram language modeling.

# Publishing new releases

Follow these steps to create a new release of Rusty DAWG.

1. Install `toml-cli` if you haven't already (`cargo install toml-cli --version 0.2.3`).
2. Run the script `./scripts/release.sh` and follow the prompts.

## Fixing a failed release

If for some reason the GitHub Actions release workflow failed with an error that needs to be fixed, you'll have to delete both the tag and corresponding release from GitHub. After you've pushed a fix, delete the tag from your local clone with

```bash
git tag -l | xargs git tag -d && git fetch -t
```

Then repeat the steps above.

# Contributions

Very welcome! There are lots of interesting algorithmic improvements under the hood to make Rusty DAWG more efficient and scalable. Get in contact if you want to help out!
