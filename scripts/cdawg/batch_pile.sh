#!/usr/bin/bash
# Create a job to process the larger or smaller subsplits of the Pile.

for split in {00..29}; do
    # This is 5x a single split.
    N_TOKENS=12603116665 \
    INPUT_PATH=original/${split}.jsonl.gz \
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
    #     INPUT_PATH="splits/${split_subsplit}.json.gz" \
    #     RUN_DIR="/output/${split_subsplit}" \
    #     beaker experiment create beaker/pile.yaml
    #     sleep 1  # Try to sleep to avoid SSH auth issues?
    # done
done