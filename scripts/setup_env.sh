#!/usr/bin/bash
# This script can be used to set up dependencies for using the Python package (and API) after a build has finished.

# First, install and setup Miniconda.
mkdir -p ~/miniconda3
wget https://repo.anaconda.com/miniconda/Miniconda3-latest-Linux-x86_64.sh -O ~/miniconda3/miniconda.sh
bash ~/miniconda3/miniconda.sh -b -u -p ~/miniconda3
rm -rf ~/miniconda3/miniconda.sh
~/miniconda3/bin/conda init bash
source /home/willm/.bashrc  # Don't re-init current shell.

# Install pip dependencies.
pip install maturin
pip install transformers
pip install flask

# Build the Python bindings.
source scripts/rebuild_bindings.sh
