#!/bin/bash
# Can call with no arguments, or like:
# ```shell
# scripts/cdawg/run_local.sh \
#     data/wikitext-2-raw/wiki.train.raw \
#     /tmp/wikitext-2-raw
# ```
# The number of tokens for Wikitext-2 is 2417786.
# The number of tokens for Wikitext-103 is 120889300 (about 50x as many).
# Larger datasets should be run with a different script most likely.

RUN_DIR=${2:-"/tmp/wikitext-2-raw"}
mkdir -p $RUN_DIR

RUST_BACKTRACE=1 ./target/release/rusty-dawg \
    --train-path ${1:-"data/wikitext-2-raw/wiki.train.raw"} \
    --n-tokens "${N_TOKENS:-2417786}" \
    --nodes-ratio 0.20 \
    --edges-ratio 1.00 \
    --tokenizer gpt2 \
    --utype u16 \
    --buf-size 1000000000 \
    --cdawg \
    --stats-threshold 10000000 \
    --stats-path "$RUN_DIR/stats.jsonl" \
    --train-vec-path "$RUN_DIR/train.vec" \
    --save-path "$RUN_DIR/cdawg" \
    --ram
