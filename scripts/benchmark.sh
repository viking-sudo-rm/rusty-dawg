#!/usr/bin/env zsh

DATA_PATH="${DATA:-data}"

# This is what was reported on StackOverflow, but it incorrectly says MB for us.
# if [[ `uname` == Darwin ]]; then
#     MAX_MEMORY_UNITS=KB
# else
#     MAX_MEMORY_UNITS=MB
# fi

TIMEFMT='================================'$'\n'\
'runtime:         %*E sec'$'\n'\
'max memory:      %M KB'

# Benchmark should be built in RAM when committed so it doesn't break CI.
time ./target/release/rusty-dawg \
    --train-path "$DATA_PATH/$1/wiki.train.raw" \
    --test-path "$DATA_PATH/$1/wiki.valid.raw" \
    --n-tokens 2051910 \
    --nodes-ratio 1.25 \
    --edges-ratio 2.20 \
    --tokenizer "whitespace" \
    --cdawg \
    #--ram \
    # --disk-path "/Users/willm/Desktop/wiki/cdawg" \
    # --train-vec-path "/Users/willm/Desktop/wiki/tokens.vec"

# Things will slow down if you don't pass a test set. Probably because all tokens are UNK.