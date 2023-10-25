import os
import subprocess

print("Downloading experiment IDs...")
experiments = os.popen("beaker group experiments willm/pile-dawg").read()
experiment_ids = []
for experiment in experiments.strip().split("\n"):
    fields = experiment.split()
    if fields[0] != "ID":
        experiment_ids.append(fields[0])

print(f"Got {len(experiment_ids)} experiment IDs:")
for id in experiment_ids:
    print(f"> beaker experiment results {id}")
procs = [subprocess.Popen(f"beaker experiment results {id}", shell=True) for id in experiment_ids]

for proc in procs:
    proc.wait()
print("All processes done!")
