#!/usr/bin/bash
# Start a screen in the background with each Pile subsplit.

DATA=/home/yanaie/data/pile/train/original
DAWGS=/home/willm/dawgs/pile/train

for split_str in {00..00}
# for split_str in {00..29}
do
    echo "Starting ${split_str}"
    screen -A -m -d -S "${split_str}" sh -c "./scripts/run_pile.sh ${DATA}/${split_str}.jsonl.gz ${DAWGS}/${split_str}"
done
