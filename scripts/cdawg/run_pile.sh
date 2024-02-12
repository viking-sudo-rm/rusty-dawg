#!/usr/bin/bash
# ========================================================================================
# Example usage: DATA=data/wikitext-2-raw N_TOKENS=2417786 source scripts/run_local.sh
# ========================================================================================
# The number of tokens for Wikitext-2 is 2417786.
# The number of tokens for Wikitext-103 is X (about 50x as many).
# Larger datasets should be run with a different script most likely.

DATA_PATH="/net/nfs.cirrascale/allennlp/willm/data/pile/00_0.json.gz"
N_TOKENS=2520623333
# TOKENIZER="EleutherAI/pythia-12b"
TOKENIZER="gpt2"
RUN_DIR="/home/willm/pile-run"

RUST_BACKTRACE=full ./target/release/rusty-dawg \
    --train-path $DATA_PATH \
    --n-tokens $N_TOKENS \
    --nodes-ratio 0.50 \
    --edges-ratio 1.50 \
    --tokenizer $TOKENIZER \
    --data-reader "pile" \
    --utype u16 \
    --buf-size 3000000000 \
    --cdawg \
    --stats-threshold 300000 \
    --stats-path "$RUN_DIR/stats.jsonl" \
    --train-vec-path "$RUN_DIR/train.vec" \
    --disk-path "$RUN_DIR/cdawg"
