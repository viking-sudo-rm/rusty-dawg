#!/usr/bin/bash
# Takes environment variables: $GCLOUD_KEY, $SPLIT, $SPLIT_TYPE

GCLOUD_PATH="lm-datasets:/mnt/tank/pile/train/$SPLIT_TYPE/$SPLIT.jsonl.gz"
SPLIT_PATH="/$SPLIT.jsonl.gz"
DISK_PATH="/output/$SPLIT"

# First, authenticate with gcloud and copy the relevant Pile shard.
gcloud auth activate-service-account --project=ai2-allennlp --key-file=$GCLOUD_KEY
gcloud compute scp --zone "us-central1-a" --recurse $GCLOUD_PATH $SPLIT_PATH

# Second, run Rusty DAWG on the downloaded Pile shard.
./target/release/rusty-dawg \
    --train-path $SPLIT_PATH \
    --disk-path $DISK_PATH \
    --n-tokens 10000000000 \
    --nodes-ratio 1.5 \
    --edges-ratio 2.3 \
    --utype "u16" \
    --tokenizer "gpt2" \
    --data-reader "pile"
