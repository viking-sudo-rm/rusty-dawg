# Rusty DAWG - Python API

Convenient API to query a DAWG and examine the results in Python.

## Usage example

```python
from transformers import GPT2Tokenizer
from api.python.dawg import PyDawg
from py_rusty_dawg import Dawg

dawg_path = [path-to-dawg]
dawg = Dawg.load(dawg_path)

# Make sure the tokenizer matches the one used to construct the DAWG.
tokenizer = GPT2Tokenizer.from_pretrained('gpt2')

py_dawg = PyDawg(dawg_path, tokenizer)

# Substring found in the Wikitext 2 train data.
query = "As with previous Valkyira Chronicles games , Valkyria Chronicles III"

# Return a list of all substrings in the DAWG that match the query.
matching_substrings = py_dawg.get_matching_substrings(query)
```
