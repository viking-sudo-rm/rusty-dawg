#!/usr/bin/bash

./target/release/rusty-dawg \
    --train-path "$1" \
    --n-tokens 10000000000 \
    --nodes-ratio 1.5 \
    --edges-ratio 2.3 \
    --utype u16 \
    --tokenizer "gpt2" \
    --data-reader "pile" \
    --disk-path "$2"
