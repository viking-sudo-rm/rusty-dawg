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

time ./target/release/rusty-dawg \
    --train-path "$DATA_PATH/$1/wiki.train.raw" \
    --test-path "$DATA_PATH/$1/wiki.valid.raw" \
    --save-path "" \
    --results-path "" \
    --n-eval 0 \
    --nodes-ratio 1.25 \
    --edges-ratio 2.20 \
    --tokenizer "whitespace"

