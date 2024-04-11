#!/bin/zsh

START=$1
END=$2
PREFIX=willm-pile2

# echo "=== Starting machines... ==="
# machines=()
# for split in {00..29}; do
#     machines+=$PREFIX-$split
# done
# gcloud compute instances start "${machines[@]}" --zone=us-central1-a
# echo "Machines started! Waiting 10 seconds to start servers."
# sleep 10

echo "\n"
echo "=== Launching servers via SSH ==="
for split in {$START..$END}; do
    machine=$PREFIX-$split
    echo "Machine $machine..."
    cmd="screen -dmS server bash -c 'cd rusty-dawg && git pull origin && ~/miniconda3/bin/python scripts/cdawg/server.py /home/willm/runs/$split --port=5000; exec bash'"
	gcloud compute ssh --zone "us-central1-a" willm@$machine --command="$cmd"
done