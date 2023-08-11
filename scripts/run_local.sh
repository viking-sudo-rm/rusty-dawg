#!/usr/bin/bash

./target/release/rusty-dawg \
    --train-path data/$1/wiki.train.raw \
    --test-path data/$1/wiki.valid.raw \
    --save-path ~/Desktop/$1.dawg \
    --results-path ~/Desktop/$1.json \
    --n-eval 0 \
    --nodes-ratio 1.25 \
    --edges-ratio 2.20 \
    --tokenize
    # -f 0 -f 1024 -f 2048 -f 4096 -f 8192 -f 16384 \
    # -d 0.01 -d 0.05 -d 0.1 -d 0.3 -d 0.5 \
    # -n 4 \
    # -i 0.9 -i 0.95 \
    # --tokenize
