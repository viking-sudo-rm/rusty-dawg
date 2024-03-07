#!/usr/bin/bash
# Create a job to process the larger or smaller subsplits of the Pile
# For the larger original splits, N_TOKENS=2520623333 * 30 and INPUT_PATH=original/${split}.jsonl.gz
# For the smaller subsplits, N_TOKENS=2520623333 and INPUT_PATH=splits/${split}_${subsplit}.json.gz

for split in {00..29}; do
    N_TOKENS=75618699990 \
    DATA_PATH=original/${split}.jsonl.gz \
    RUN_DIR="/output/${split}" \
    beaker experiment create beaker/pile.yaml
    sleep 1
    # ==========
    # Subsplits
    # ==========
    # for subsplit in {0..4}; do
    #     split_subsplit="${split}_${subsplit}"
    #     echo "Starting ${split_subsplit}"
    #     N_TOKENS=2520623333 \
    #     DATA_PATH="splits/${split_subsplit}.json.gz" \
    #     RUN_DIR="/output/${split_subsplit}" \
    #     beaker experiment create beaker/pile.yaml
    #     sleep 1  # Try to sleep to avoid SSH auth issues?
    # done
done