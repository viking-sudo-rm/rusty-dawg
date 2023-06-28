#!/usr/bin/bash

./target/release/rusty-dawg \
    --train-path /Users/willm/Desktop/wikitext-2-raw/wiki.train.raw \
    --test-path /Users/willm/Desktop/wikitext-2-raw/wiki.valid.raw \
    --save-path /Users/willm/Desktop/wikitext2.dawg \
    --results-path /Users/willm/Desktop/wikitext2.json \
    --tokenize
