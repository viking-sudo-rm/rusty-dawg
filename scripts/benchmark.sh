#!/usr/bin/bash

./target/release/rusty-dawg \
    --train-path data/$1/wiki.train.raw \
    --test-path data/$1/wiki.valid.raw \
    --save-path "/tmp/$1.dawg" \
    --results-path "" \
    --n-eval 0 \
    --tokenize
