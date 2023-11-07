import os
import re
import subprocess
import argparse
import yaml

SPLIT_PATTERN = r'\d{2}_\d{1}'

parser = argparse.ArgumentParser()
parser.add_argument("--manual_splits", type=str, default=None)
parser.add_argument("--clear_existing", action="store_true")
args = parser.parse_args()

def download_experiment_ids():
    experiments = os.popen("beaker group experiments willm/pile-dawg").read()
    experiment_ids = []
    for experiment in experiments.strip().split("\n"):
        fields = experiment.split()
        if fields[0] != "ID":
            experiment_ids.append(fields[0])
    return experiment_ids

def load_splits(path):
    with open(path) as fh:
        return fh.read().strip().split()

print("Downloading experiment list...")
experiment_ids = download_experiment_ids()
manual_splits = set(load_splits(args.manual_splits)) if args.manual_splits is not None else set([])
print(f"Got {len(experiment_ids)} experiments.")

if not manual_splits and os.listdir("build-dawg"):
    print("Attempting to download everything and there is data already!")
    print("Exiting!")
    exit()

print("Retrieving experiment specs...")
splits = []
for id in experiment_ids:
    spec = os.popen(f"beaker experiment spec {id}").read()
    spec = yaml.load(spec, Loader=yaml.FullLoader)
    description = spec["description"]
    split = re.findall(SPLIT_PATTERN, description)[0]
    if manual_splits and split not in manual_splits:
        continue
    splits.append(split)

print(f"Splits ({len(splits)}):", ", ".join(splits))
if input(f"Confirm delete these splits? [y]?") != "y":
    print("Exiting!")
    exit()

print("Deleting existing splits...")
for split in splits:
    print(f"> Deleting build-dawg/{split}...")
    os.system(f"rm -rf build-dawg/{split}")

print("Downloading all splits...")
procs = [subprocess.Popen(f"beaker experiment results {id}", shell=True) for id in experiment_ids]
for proc in procs:
    proc.wait()
print("All processes done!")
