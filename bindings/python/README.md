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