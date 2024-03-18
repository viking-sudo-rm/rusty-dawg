#!/usr/bin/bash
# Takes environment variables: $N_TOKENS, $INPUT_PATH, $OUTPUT_PATH
set -xe

GCLOUD_PATH="lm-datasets:/mnt/tank/pile/train/$INPUT_PATH"

# First, authenticate with gcloud.
gcloud auth activate-service-account --project=ai2-allennlp --key-file=$GCLOUD_KEY

# Copy the relevant pile shard.
echo "Copying Pile shard: $GCLOUD_PATH"
gcloud compute scp --zone "us-central1-a" $GCLOUD_PATH /data.jsonl.gz

# Set allocation variables for Pythia tokenizer (change if different tokenizer).
NODES_RATIO=0.18
EDGES_RATIO=0.98

# Run Rusty DAWG on the downloaded Pile shard.
./target/release/rusty-dawg \
    --train-path /data.jsonl.gz \
    --n-tokens $N_TOKENS \
    --nodes-ratio $NODES_RATIO \
    --edges-ratio $EDGES_RATIO \
    --cache-size 0 \
    --buf-size 3000000000 \
    --tokenizer "EleutherAI/pythia-12b" \
    --data-reader "pile" \
    --utype u16 \
    --cdawg \
    --stats-threshold 100000000 \
    --stats-path "$RUN_DIR/stats.jsonl" \
    --train-vec-path "$RUN_DIR/train.vec" \
    --disk-path "$RUN_DIR/cdawg"

# Counts are being built in RAM here. If it fails, can add --no-counts and add later.
