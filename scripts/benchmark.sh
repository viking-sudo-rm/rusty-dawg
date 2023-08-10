#!/usr/bin/zsh

DATA_PATH="${DATA:-data}"

# This is what was reported on StackOverflow, but it incorrectly says MB for us.
# if [[ `uname` == Darwin ]]; then
#     MAX_MEMORY_UNITS=KB
# else
#     MAX_MEMORY_UNITS=MB
# fi

TIMEFMT='max memory:      %M KB'

command time --format "$TIMEFMT" ./target/release/rusty-dawg \
    --train-path "$1" \
    --test-path "$2" \
    --save-path "" \
    --results-path "" \
    --n-eval 0 \
    --tokenize

# size=$(ls -lh /tmp/$1.dawg | awk '{print  $5}')
# rm /tmp/$1.dawg
# echo "\n=====\nsize: $size"
