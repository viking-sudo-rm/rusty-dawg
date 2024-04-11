#!/usr/bin/zsh
# Create a job to process the larger or smaller subsplits of the Pile.

START=$1
END=$2
PREFIX=willm-pile2
echo "Running from $START to $END..."

for split in {$START..$END}; do
    MACHINE=$PREFIX-$split
    echo "=== Creating $MACHINE... ==="
    gcloud compute instances create $MACHINE --source-instance-template willm-ram-384gb --zone "us-central1-a"
done

for split in {$START..$END}; do
    MACHINE=$PREFIX-$split
    echo "=== Launching $MACHINE... ==="
    gcloud compute scp --zone "us-central1-a" --recurse /home/willm/rusty-dawg-startup $MACHINE:/home/willm/rusty-dawg-startup
    cmd="screen -dmS cdawg bash -c 'export SPLIT=$split && chmod +x rusty-dawg-startup/wrap_startup.sh && sudo shutdown -h now; exec bash'"
    echo "Running command: $cmd"
    gcloud compute ssh --zone "us-central1-a" willm@$MACHINE --command="$cmd"
done