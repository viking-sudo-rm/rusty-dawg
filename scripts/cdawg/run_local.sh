#!/bin/bash
# ========================================================================================
# Example usage: DATA=data/wikitext-2-raw N_TOKENS=2417786 source scripts/run_local.sh
# ========================================================================================
# The number of tokens for Wikitext-2 is 2417786.
# The number of tokens for Wikitext-103 is X (about 50x as many).
# Larger datasets should be run with a different script most likely.

DATA_PATH="${DATA:-data/wikitext-2-raw}"
N_TOKENS="${N_TOKENS:-2417786}"
RUN_DIR=${2:-"/tmp/wiki-cdawg"}

# Currently need to do this in advance for the RAM -> disk case.
mkdir -p $RUN_DIR

RUST_BACKTRACE=1 ./target/release/rusty-dawg \
    --train-path "$DATA_PATH/$1/wiki.train.raw" \
    --n-tokens $N_TOKENS \
    --nodes-ratio 0.20 \
    --edges-ratio 1.00 \
    --tokenizer gpt2 \
    --utype u16 \
    --buf-size 1000000000 \
    --cdawg \
    --stats-threshold 10000000 \
    --stats-path "$RUN_DIR/stats.jsonl" \
    --train-vec-path "$RUN_DIR/train.vec" \
    --disk-path "$RUN_DIR/cdawg" \
    --ram
