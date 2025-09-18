#!/usr/bin/bash

# ./target/release/rusty-dawg \
#     --train-path "$1" \
#     --n-tokens 10000000000 \
#     --nodes-ratio 1.5 \
#     --edges-ratio 2.3 \
#     --utype u16 \
#     --tokenizer "gpt2" \
#     --data-reader "pile" \
#     --save-path "$2"

DATA_PATH=${1:-"/net/nfs.cirrascale/allennlp/willm/data/pile/00_0.json.gz"}
RUN_DIR=${2:-"/home/willm/pile-run"}

# Allocation variables, based on Pythia tokenizer. Added 0.01 for good measure!
NODES_RATIO=0.19
EDGES_RATIO=0.99
N_TOKENS=12603116665 # 2520623333

./target/release/rusty-dawg \
    --train-path $DATA_PATH \
    --n-tokens $N_TOKENS \
    --nodes-ratio $NODES_RATIO \
    --edges-ratio $EDGES_RATIO \
    --cache-size 0 \
    --buf-size 3000000000 \
    --tokenizer "EleutherAI/pythia-12b" \
    --data-reader "pile" \
    --utype u16 \
    --save-path "$RUN_DIR/dawg"