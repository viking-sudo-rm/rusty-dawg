#!/usr/bin/bash

N_TOKENS=1000
OUT_DIR=/net/nfs/allennlp/willm/lm-generations

# echo "========== Generating data from different models =========="
# N_SEEDS=50
# SIZES=("70m" "160m" "410m" "1b" "1.4b" "2.8b" "6.9b" "12b")
# BATCH_SIZES=("50" "50" "50" "50" "50" "50" "20" "10")
# mkdir $OUT_DIR/models

# for idx in "${!SIZES[@]}"; do
#     size=${SIZES[idx]}
#     batch_size=${BATCH_SIZES[idx]}

#     MODEL="pythia-$size"
#     echo "===== ${MODEL} ====="
#     python scripts/generate_from_lm.py \
#         EleutherAI/${MODEL} \
#         $OUT_DIR/models/${MODEL}.jsonl \
#         --n_tokens=$N_TOKENS \
#         --n_seeds=$N_SEEDS \
#         --batch_size=$batch_size \
#         --sample
#     MODEL="pythia-$size-deduped"
#     echo "===== ${MODEL} ====="
#     python scripts/generate_from_lm.py \
#         EleutherAI/${MODEL} \
#         $OUT_DIR/models/${MODEL}.jsonl \
#         --n_tokens=$N_TOKENS \
#         --n_seeds=$N_SEEDS \
#         --batch_size=$batch_size \
#         --sample
# done

# rm $OUT_DIR/models.jsonl
# cat $OUT_DIR/models/*.jsonl > $OUT_DIR/models.jsonl


echo "========== Generating decoding data =========="
mkdir $OUT_DIR/decoding

MODELS=("pythia-12b" "pythia-12b-deduped")
for MODEL in "${MODELS[@]}"; do
    echo "===== ${MODEL} ====="
    python scripts/generate_from_lm.py \
        EleutherAI/${MODEL} \
        $OUT_DIR/decoding/${MODEL}.jsonl \
        --n_tokens=$N_TOKENS \
        --n_seeds=10 \
        --batch_size=10 \
        -k 20 40 80 160 320 \
        -p 0.85 0.90 0.95 1.00 \
        -t 0.85 0.90 0.95 1.00 1.05 2.00 \
        -b 1 2 4 6
done

rm $OUT_DIR/decoding.jsonl
cat $OUT_DIR/decoding/*.jsonl > $OUT_DIR/decoding.jsonl
