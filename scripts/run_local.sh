#!/usr/bin/bash

DATA_PATH="${DATA:-data}"

./target/release/rusty-dawg \
    --train-path "$DATA_PATH/$1/wiki.train.raw" \
    --test-path "$DATA_PATH/$1/wiki.valid.raw" \
    --save-path "" \
    --results-path "" \
    --n-eval 0 \
    --nodes-ratio 1.25 \
    --edges-ratio 2.20 \
    --disk-path "/tmp/$1-dawg" \
    --tokenizer gpt2 \
    --utype u16
