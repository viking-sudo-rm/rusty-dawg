#!/usr/bin/bash
# Takes environment variables: $N_TOKENS, $INPUT_PATH, $OUTPUT_PATH
set -xe

GCLOUD_PATH="lm-datasets:/mnt/tank/pile/train/$INPUT_PATH"

# First, authenticate with gcloud.
gcloud auth activate-service-account --project=ai2-allennlp --key-file=$GCLOUD_KEY

# Copy the relevant pile shard.
echo "Copying Pile shard: $GCLOUD_PATH"
gcloud compute scp --zone "us-central1-a" --recurse $GCLOUD_PATH /data.jsonl.gz

# Run Rusty DAWG on the downloaded Pile shard.
./target/release/rusty-dawg \
    --train-path /data.jsonl.gz \
    --disk-path $OUTPUT_PATH \
    --n-tokens $N_TOKENS \
    --nodes-ratio 1.5 \
    --edges-ratio 2.3 \
    --utype "u16" \
    --tokenizer "gpt2" \
    --data-reader "pile"
