#!/usr/bin/bash
# Spin up VM instances for splits

# FIXME: Change numbers here for what you need.
for idx in {3..29}; do
    gcloud compute instances create willm-cdawg-500gb-$idx \
        --source-instance-template willm-cdawg-500gb \
        --zone us-central1-a
done

# TODO: SSH to them, run set up, start building in screen, etc.
# I'm just doing this manually right now.