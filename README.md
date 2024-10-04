# Rusty-DAWG

A library for building directed acyclic word graphs (DAWGs) or compacted DAWGS (CDAWGS) for string indexing and searching in Rust.

The key features are:
1. Build a DAWG or CDAWG indexon a corpus with a one-liner. The (C)DAWG can be saved in two formats: a graph stored in RAM and a graph stored on disk.
2. Use Python bindings to load a saved (C)DAWG for doing fast n-gram search (you can also load it in Rust, but we recommend working with the Python API).
3. An API server and web demo for visualizing n-gram search results over a pre-built (C)DAWG.

## Authors

This library was started by Will Merrill and Yanai Elazar as part of an internship project at AI2. Ananya Jha, Rodney Kinney, David Wadden, Pete Walsh have all since contributed core engineering features. We've also appreciated the support of Michal Guerquin, Johann Dahm, and other members of the Beaker team at AI2 for getting the library to run at very large scale, as well as the data structures expertise of Shunsuke Inenaga.

# Getting Started

## Installing Rust

Simply use the one-liner [here](https://www.rust-lang.org/tools/install).

## Testing and Building Rusty-DAWG

To run tests, you can call Cargo (which should have been installed with Rust) from inside the repo directory:

```bash
cargo test
```

To compile an optimized release build, you can run:

```bash
cargo build --release
```

Note that the `--release` flag is very important for performance. The code will be 10-100x slower without it.

## Running Benchmarking Script

To run the benchmarking script, you need the Wikitext2/103 data. You can either download this to rusty-dawg/data path or point to an existing repository (easy on beaker, you can use my copy of the data).

You first need to download the [data](https://drive.google.com/file/d/1XRZA2eki_Z8M0QrYN4BrbN7dghMYqYby/view?usp=sharing) directory, unzip it, and put it in the root of the repository directory (i.e., rusty-dawg/data). Then you can run:

```bash
./scripts/benchmark.sh wikitext-2-raw
```

If the data is stored somewhere else, you can do:

```bash
DATA=/home/willm/splits ./scripts/benchmark.sh wikitext-2-raw
```

<!-- The benchmarking spreadsheet requests both the runtime and the memory overhead. The total runtime will be printed out by the script's progress bar. The benchmarking script will also print out the size of the DAWG at the bottom. -->

# Building Your CDAWG

The core functionality of Rusty-DAWG is to build DAWGs and CDAWGs, which are indexing structures for large corpora. The CDAWG is a strict improvement of the DAWG, so we recommend using the CDAWG if you are building a new index from scratch.

To get started building a CDAWG on your corpus, we recommend adapting the [scripts/cdawg/run_pile.sh](https://github.com/viking-sudo-rm/rusty-dawg/blob/main/scripts/cdawg/run_pile.sh) script. This script was written to build a CDAWG (memory-efficient improvement of DAWG) on the Pile. It could be run as follows:

```bash
scripts/cdawg/run_pile.sh /home/willm/data/pile/00_0.json.gz /home/willm/cdawgs/00_0
```

Here the first argument is the path where the input corpus can be found.
By default, this should be a path to an input file in `.jsonl.gz` format, where each line looks like:

```
{"text": "this is a document", "meta": {"data": "here"}}
```

The `meta` key must be present but `"meta": {}` can be specified if no metadata exists. If you wish to pass input data in a different format, you can change the ``--data-reader`` flag to a different option.

The second argument is a path at which output CDAWG will get created (as well as a disk vector storing a copy of the training tokens and a log of CDAWG stats during building).

Other arguments in the [scripts/cdawg/run_pile.sh](https://github.com/viking-sudo-rm/rusty-dawg/blob/main/scripts/cdawg/run_pile.sh) script:

* `N_TOKENS`, `NODES_RATIO`, and `EDGES_RATIO`: These are used to allocate memory for the CDAWG. `N_TOKENS` should be an upper bound on the number of tokens in the dataset. `NODES_RATIO` and `EDGES_RATIO` should be upper bounds on the # of nodes and # of edges per input token. For the DAWG, these have an upper bound of 2 and 3, and for the CDAWG, they will typically be (well) below 1 and 2. You can estimate these values for a large dataset by simply building on a smaller chunk of the data first and extrapolating.
* Tokenizer: By default, this script uses the `gpt2` tokenizer. You might consider using a different tokenizer, since `gpt2` treats whitespace somewhat poorly.
* Cache size: This parameters simply controls how many bytes of text are read into RAM at once while decompressing the training data. It isn't that important, but if you run into RAM issues, you should lower it!

# Using CDAWGs for Inference in Python

The library is implemented in Rust, but DAWGs, once built, can be loaded and used easily in Python! You can even build DAWGs from scratch using the Python bindings, though we don't necessarily recommend that.

## Building the Python Bindings

The Python bindings are generated using [maturin](https://github.com/PyO3/maturin). First install maturin in your Python environment:

```bash
pip install maturin
```

Then, you should be able to build (or rebuild) the Python bindings with:

```bash
source scripts/rebuild_bindings.sh
```

*(If above doesn't work)* Alternatively, `cd` into the Python bindings directory (`bindings/python`) and run:

```bash
make install
```

*(If above still doesn't work)* You can build the bindings in two steps:

```bash
python -m maturin build --release
pip install target/wheels/*.whl
```

## Using the Python Library

After installing the bindings, you should be able to import the library:

```python
from rusty_dawg import Cdawg, DiskCdawg
```

Refer to [scripts/cdawg/test_cdawg_matches_dawg.py](https://github.com/viking-sudo-rm/rusty-dawg/blob/main/scripts/cdawg/test_cdawg_matches_dawg.py) for an example of how to build and use a CDAWG in RAM with the Python bindings. To use a disk CDAWG instead, you can use `DiskCdawg` instead of `Cdawg`. [scripts/cdawg/test_load_cdawg.py](https://github.com/viking-sudo-rm/rusty-dawg/blob/main/scripts/cdawg/test_load_cdawg.py) shows an example of how to load a pre-built `DiskCdawg`.

# Citation

```bibtex
@misc{merrill2024evaluatingngramnoveltylanguage,
      title={Evaluating $n$-Gram Novelty of Language Models Using Rusty-DAWG},
      author={William Merrill and Noah A. Smith and Yanai Elazar},
      year={2024},
      eprint={2406.13069},
      archivePrefix={arXiv},
      primaryClass={cs.CL}
      url={https://arxiv.org/abs/2406.13069},
}
```
