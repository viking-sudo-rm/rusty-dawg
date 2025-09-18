#!/usr/bin/bash
# ========================================================================================
# Example usage: DATA=data/wikitext-2-raw N_TOKENS=2417786 source scripts/run_local.sh
# ========================================================================================
# The number of tokens for Wikitext-2 is 2417786.
# The number of tokens for Wikitext-103 is X (about 50x as many).
# Larger datasets should be run with a different script most likely.

DATA_PATH="${DATA:-data/wikitext-2-raw}"
N_TOKENS="${N_TOKENS:-2417786}"

RUST_BACKTRACE=1 ./target/release/rusty-dawg \
    --train-path "$DATA_PATH/$1/wiki.train.raw" \
    --n-tokens $N_TOKENS \
    --nodes-ratio 1.25 \
    --edges-ratio 2.20 \
    --tokenizer gpt2 \
    --utype u16 \
    --buf-size 1000000000 \
    --save-path "/Users/willm/Desktop/run-local/dawg"
