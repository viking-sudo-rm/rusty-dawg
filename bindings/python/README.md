# (Py) Rusty DAWG

A Python wrapper for [Rusty DAWG](https://github.com/viking-sudo-rm/rusty-dawg), providing seamless access to fast and memory-efficient DAWG data structures implemented in Rust.

## Building

First update Rusty DAWG to the most recent version from the GitHub repo:

```
cargo update -p rusty-dawg
```

Then build via Maturin:

```
maturin build --release
# python3 -m maturin build --release
```

To update the Git repo dependencies:

```
cargo update
```

Finally, install the generated wheel via pip in your Python installation:

```
pip install target/wheels/*.whl
```

## Usage example

```python
from transformers import GPT2Tokenizer
from rusty_dawg import Dawg, PyDawg

dawg_path = [path-to-dawg]
dawg = Dawg.load(dawg_path)

# Make sure the tokenizer matches the one used to construct the DAWG.
tokenizer = GPT2Tokenizer.from_pretrained('gpt2')

py_dawg = PyDawg(dawg, tokenizer)

# Substring found in the Wikitext 2 train data.
query = "As with previous Valkyira Chronicles games , Valkyria Chronicles III"

# Return a list of all substrings in the DAWG that match the query.
matching_substrings = py_dawg.get_matching_substrings(query)
```
