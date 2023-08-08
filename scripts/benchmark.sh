#!/usr/bin/bash

DATA_PATH="${DATA:-data}"

./target/release/rusty-dawg \
    --train-path $DATA_PATH/$1/wiki.train.raw \
    --test-path $DATA_PATH/$1/wiki.valid.raw \
    --save-path "/tmp/$1.dawg" \
    --results-path "" \
    --n-eval 0 \
    --tokenize