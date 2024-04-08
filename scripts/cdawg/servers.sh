#!/usr/bin/bash

# Function to cleanup background processes
cleanup() {
    echo "Cleaning up..."
    for pid in "${pids[@]}"; do
        if ps -p $pid > /dev/null; then
            kill "$pid"
        fi
    done
}
trap cleanup EXIT

PREFIX=willm-pile2

machines=()
for split in {00..29}; do
    machines+=$PREFIX-$split
done
gcloud compute instances start "${machines[@]}" --zone=us-central1-a

echo "Machines started! Waiting 10 seconds to start servers."
sleep 10

pids=()
for split in {00..29}; do
    MACHINE=$PREFIX-$split
    echo "\n === Machine $MACHINE ==="

    # SSH and start server in screen
    cmd="screen -dmS cdawg bash -c 'cd rusty-dawg && git pull origin && ~/miniconda3/bin/python scripts/cdawg/server.py /home/willm/runs/$split --port=5000; exec bash'"
	gcloud compute ssh --zone "us-central1-a" willm@$MACHINE --command="$cmd"
    
    # Create IAP tunnel.
    local_port=$(echo "ibase=10; 5000 + $split" | bc)
    gcloud compute start-iap-tunnel willm-pile2-00 5000 \
        --local-host-port=localhost:$local_port \
        --zone=us-central1-a &
    pids+=($!)
done

echo "IAP tunnels created; waiting to exit..."
for pid in "${pids[@]}"; do
    wait "$pid"
done