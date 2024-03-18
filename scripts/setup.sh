#!/usr/bin/bash
# Setup a new VM to run Rusty DAWG

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# TODO: Probably need to rerun setup script or something here
cargo build --release

DATA_PATH=$1
LOCAL_DIR=/home/willm/data

# Login to Google Cloud and download the data
gcloud auth login
gcloud compute scp --zone "us-central1-a" "lm-datasets:/mnt/tank/pile/train/$DATA_PATH" $LOCAL_DIR/data.jsonl.gz