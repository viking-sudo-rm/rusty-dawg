#!/bin/bash
# This script can be used to set up dependencies for using the Python package (and API) after a build has finished.

# First, install and setup Miniconda.
mkdir -p ~/miniconda3
wget https://repo.anaconda.com/miniconda/Miniconda3-latest-Linux-x86_64.sh -O ~/miniconda3/miniconda.sh
bash ~/miniconda3/miniconda.sh -b -u -p ~/miniconda3
rm -rf ~/miniconda3/miniconda.sh
~/miniconda3/bin/conda init bash
source ~/.bashrc  # Reconfigure current shell.

# FIXME: When ssh'ing, pip/python don't work.
echo "Path after running ~/.bashrc:"
echo $PATH

# Install pip dependencies.
~/miniconda3/bin/python -m pip install maturin
~/miniconda3/bin/python -m pip install transformers
~/miniconda3/bin/python -m pip install flask

# Build the Python bindings.
PYTHON=~/miniconda3/bin/python source scripts/rebuild_bindings.sh
