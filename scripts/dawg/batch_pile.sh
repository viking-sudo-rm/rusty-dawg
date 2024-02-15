#!/usr/bin/bash
# Create a job to process the larger or smaller subsplits of the Pile
# For the larger original splits, N_TOKENS=10B and INPUT_PATH=original/${split}.jsonl.gz
# For the smaller subsplits, N_TOKENS=2B and INPUT_PATH=splits/${split}_${subsplit}.json.gz

# for split in {00..29}; do
#     for subsplit in {0..4}; do
#         split_subsplit="${split}_${subsplit}"
#         echo "Starting ${split_subsplit}"
#         N_TOKENS=2000000000 \
#         INPUT_PATH="splits/${split_subsplit}.json.gz" \
#         OUTPUT_PATH="/output/${split_subsplit}" \
#         beaker experiment create beaker/pile.yaml
#         sleep 1  # Try to sleep to avoid SSH auth issues?
#     done
# done

# Manual restarts on October 19.
failed=(
    29_3
    29_2
    29_1
    28_4
    28_1
    27_4
    27_2
    27_0
    26_4
    25_4
    24_4
    24_3
    24_2
    24_1
    23_4
    23_3
    23_2
    23_1
    23_0
    22_4
    22_3
    22_2
    21_4
    21_0
    20_2
    20_1
    20_0
    19_4
    19_1
    19_0
    18_4
    18_1
    18_0
    17_4
    17_3
    17_1
    17_0
    16_0
    15_4
    15_0
    14_4
    14_3
    14_2
    14_1
    14_0
    13_3
    13_2
    13_1
    13_0
    12_4
    12_2
    12_1
    11_4
    11_3
    11_2
    10_4
    09_3
    09_0
    08_4
    08_3
    08_2
    08_1
    07_4
    07_2
    07_0
    06_4
    06_3
    06_2
    06_1
    06_0
    05_4
    05_3
    05_2
    05_1
    05_0
    04_4
    04_1
    04_0
    03_4
    03_3
    03_0
    01_4
    01_1
    00_3
)


for split_subsplit in "${failed[@]}"; do
    echo "Starting ${split_subsplit}"
    N_TOKENS=2000000000 \
    INPUT_PATH="splits/${split_subsplit}.json.gz" \
    OUTPUT_PATH="/output/${split_subsplit}" \
    beaker experiment create beaker/pile.yaml
    sleep 1  # Try to sleep to avoid SSH auth issues?
done