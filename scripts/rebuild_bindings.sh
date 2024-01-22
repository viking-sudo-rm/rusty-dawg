#!/bin/zsh

cd bindings/python
rm -rf target/wheels
python -m maturin build --release
python -m pip install target/wheels/* --ignore-installed
cd ../..