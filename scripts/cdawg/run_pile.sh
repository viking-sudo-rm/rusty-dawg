#!/usr/bin/bash

DATA_PATH=${1:-"/net/nfs.cirrascale/allennlp/willm/data/pile/00_0.json.gz"}
RUN_DIR=${2:-"/home/willm/pile-run"}

# Currently need to do this in advance for the RAM -> disk case.
mkdir -p $RUN_DIR

# Allocation variables, based on Pythia tokenizer. Added 0.01 for good measure!
N_TOKENS=${N_TOKENS:-11117142449} # 2520623333
NODES_RATIO=0.19
EDGES_RATIO=0.98

./target/release/rusty-dawg \
    --train-path $DATA_PATH \
    --n-tokens $N_TOKENS \
    --nodes-ratio $NODES_RATIO \
    --edges-ratio $EDGES_RATIO \
    --cache-size ${CACHE_SIZE:-0} \
    --buf-size 3000000000 \
    --tokenizer "EleutherAI/pythia-12b" \
    --data-reader "pile" \
    --utype u16 \
    --cdawg \
    --stats-threshold 10000000 \
    --stats-path "$RUN_DIR/stats.jsonl" \
    --train-vec-path "$RUN_DIR/train.vec" \
    --save-path "$RUN_DIR/cdawg" \
    --ram
