#!/bin/zsh

PYTHON=${PYTHON:-python}

cd bindings/python
rm -rf target/wheels
$PYTHON -m maturin build --release
$PYTHON -m pip install target/wheels/* --ignore-installed
cd ../..