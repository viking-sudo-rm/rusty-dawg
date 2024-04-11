#!/bin/zsh

START=$1
END=$2
PREFIX=willm-pile2

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

pids=()
for split in {$START..$END}; do
    machine=$PREFIX-$split
    local_port=$(echo "ibase=10; 5000 + $split" | bc)
    gcloud compute start-iap-tunnel $machine 5000 \
        --local-host-port=localhost:$local_port \
        --zone=us-central1-a &
    pids+=($!)
done

echo "IAP tunnels created; waiting to exit..."
for pid in "${pids[@]}"; do
    wait "$pid"
done