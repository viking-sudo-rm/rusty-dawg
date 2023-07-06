#!/usr/bin/bash

CORPUS=$1

# Escape spaces for corpus names like Wikipedia (en)
SPLIT="/home/willm/splits/$CORPUS"
RESULTS=/home/willm/results

./target/release/rusty-dawg \
    --train-path "$SPLIT/train.txt" \
    --test-path "$SPLIT/val.txt" \
    --save-path "$RESULTS/$CORPUS.dawg" \
    --results-path "$RESULTS/$CORPUS.json" \
    --gen-path "$SPLIT/gpt2.txt" \
    --gen-results-path "$RESULTS/$CORPUS-gpt2.json" \
    --max_length 500 \  #Max length for evaluating histogram of suffix lengths.
    -f 0 -f 1024 -f 2048 -f 4096 -f 8192 \
    -d 0.01 -d 0.05 -d 0.1 -d 0.3 -d 0.5 -d 0.7 \
    -n 4 \
    -i 0.95
