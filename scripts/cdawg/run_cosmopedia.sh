#!/usr/bin/bash
# TODO: Merge script for Pile/Cosmopedia runs.

DATA_PATH=${1:-"/net/nfs.cirrascale/allennlp/willm/data/cosmopedia/cat.jsonl.gz"}
RUN_DIR=${2:-"/net/nfs.cirrascale/allennlp/willm/cdawgs/cosmopedia"}

# Currently need to do this in advance for the RAM -> disk case.
mkdir -p $RUN_DIR

# Allocation variables, based on Pythia tokenizer. Added 0.01 for good measure!
N_TOKENS=${N_TOKENS:-26000000000}  # Cosmopedia is at least 25B tokens.
NODES_RATIO=0.2
EDGES_RATIO=1.1
# TODO: Validate these choices

./target/release/rusty-dawg \
    --train-path $DATA_PATH \
    --n-tokens $N_TOKENS \
    --nodes-ratio $NODES_RATIO \
    --edges-ratio $EDGES_RATIO \
    --cache-size ${CACHE_SIZE:-0} \
    --buf-size 3000000000 \
    --tokenizer "HuggingFaceTB/cosmo-1b" \
    --data-reader "jsonl" \
    --utype u16 \
    --cdawg \
    --stats-threshold 10000000 \
    --stats-path "$RUN_DIR/stats.jsonl" \
    --train-vec-path "$RUN_DIR/train.vec" \
    --save-path "$RUN_DIR/cdawg" \
    --ram
