#!/usr/bin/bash
# Setup a new VM to run Rusty DAWG

sudo apt install build-essential

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
. "$HOME/.cargo/env"  # Reload setup script to find cargo
cargo build --release

DATA_PATH=$1
LOCAL_DIR=/home/willm/data
mkdir $LOCAL_DIR

# Login to Google Cloud and download the data
gcloud auth login
gcloud compute scp --zone "us-central1-a" "lm-datasets:/mnt/tank/pile/train/$DATA_PATH" $LOCAL_DIR/data.jsonl.gz