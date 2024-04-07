#!/usr/bin/bash

for split in {00..29}; do
    MACHINE=$PREFIX-$split
    gcloud compute instances start $MACHINE --zone=us-central1-a
    # TODO: SSH and start server in screen
    # TODO: Create IAP tunnel
done