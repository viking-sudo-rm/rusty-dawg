# Contributing to Rusty-DAWG

Thank you for your interest in contributing to Rusty-DAWG!
There are lots of interesting algorithmic improvements under the hood to make Rusty-DAWG more efficient and scalable. Get in contact if you want to help out!

This file provides some high-level documentation about the library and guidance for code style and deployments.

## Core Packages

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

# Code style

Before contributing code, make sure to run format and clippy:
```shell
cargo clippy --fix
cargo fmt --
```

To test for possible other clippy issues, run:

```shell
cargo clippy --all-targets -- -D warnings \
    -A clippy::comparison_chain \
    -A clippy::upper-case-acronyms \
    -A dead-code
```

# Publishing New Releases

Follow these steps to create a new release of Rusty-DAWG.

1. Install `toml-cli` if you haven't already (`cargo install toml-cli --version 0.2.3`).
2. Run the script `./scripts/release.sh` and follow the prompts.

## Fixing a Failed Release

If for some reason the GitHub Actions release workflow failed with an error that needs to be fixed, you'll have to delete both the tag and corresponding release from GitHub. After you've pushed a fix, delete the tag from your local clone with

```bash
git tag -l | xargs git tag -d && git fetch -t
```

Then repeat the steps above.