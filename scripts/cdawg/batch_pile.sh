#!/usr/bin/bash
# Create a job to process the larger or smaller subsplits of the Pile.

# FIXME: Change starts back to 00

for split in {21..29}; do
    MACHINE=willm-pile-$split
    echo "Creating $MACHINE..."
    gcloud compute instances create $MACHINE --source-instance-template willm-ram-384gb --zone "us-central1-a"
done

for split in {21..29}; do
    MACHINE=willm-pile-$split
    gcloud compute scp --zone "us-central1-a" --recurse /home/willm/rusty-dawg-startup $MACHINE:/home/willm/rusty-dawg-startup
    cmd="screen -dmS cdawg bash -c 'export SPLIT=$split && chmod +x rusty-dawg-startup/wrap_startup.sh && rusty-dawg-startup/wrap_startup.sh; exec bash'"
    echo "Running command: $cmd"
    gcloud compute ssh --zone "us-central1-a" willm@$MACHINE --command="$cmd"
done