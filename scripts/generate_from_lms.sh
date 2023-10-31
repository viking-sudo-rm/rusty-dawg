#!/usr/bin/bash

N_TOKENS=1000
N_SEEDS=50

SIZES=("70m" "160m" "410m" "1b" "1.4b" "2.8b" "6.9b" "12b")
BATCH_SIZES=("50" "50" "50" "50" "50" "50" "20" "10")

OUT_DIR=/net/nfs/allennlp/willm/lm-generations
SCRATCH=/net/nfs/allennlp/willm/lm-generations/scratch

mkdir $SCRATCH

for idx in "${!SIZES[@]}"; do
    size=${SIZES[idx]}
    batch_size=${BATCH_SIZES[idx]}

    MODEL="pythia-$size"
    echo "===== ${MODEL} ====="
    python scripts/generate_from_lm.py \
        EleutherAI/${MODEL} \
        $SCRATCH/${MODEL}.jsonl \
        --n_tokens=$N_TOKENS \
        --n_seeds=$N_SEEDS \
        --batch_size=$batch_size
    MODEL="pythia-$size-deduped"
    echo "===== ${MODEL} ====="
    python scripts/generate_from_lm.py \
        EleutherAI/${MODEL} \
        $SCRATCH/${MODEL}.jsonl \
        --n_tokens=$N_TOKENS \
        --n_seeds=$N_SEEDS \
        --batch_size=$batch_size
done

# Combine all the lines in one file.
cat $SCRATCH/*.jsonl > $OUT_DIR/all.jsonl
