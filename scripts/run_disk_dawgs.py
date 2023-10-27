import argparse
import subprocess
from tqdm import tqdm
import os
import json
from transformers import AutoTokenizer
import numpy as np

from rusty_dawg import Dawg, DiskDawg

scratch_dir = "/home/willm/scratch"

COMMAND = """python3 scripts/run_disk_dawg.py \
    --dawg_dir={dawg_dir} \
    --data_path={data_path} \
    --handle_exceptions \
    --no_print \
    --npy_path={npy_path} \
    --n_lines=1
"""

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--dawgs_dir", type=str)
    parser.add_argument("--data_path", type=str)
    parser.add_argument("--n_lines", type=int, default=10000)
    args = parser.parse_args()

    procs = []
    for split in os.listdir(args.dawgs_dir):
         dawg_dir = os.path.join(args.dawgs_dir, split)
         npy_path = os.path.join(scratch_dir, split + ".npy")
         command = COMMAND.format(dawg_dir=dawg_dir, data_path=args.data_path, npy_path=npy_path)
         procs.append(subprocess.Popen(command, shell=True))

    for proc in procs:
        proc.wait()
    print("All processes done!")

    all_lengths = []
    for split in os.listdir(args.dawgs_dir):
        npy_path = os.path.join(scratch_dir, split + ".npy")
        lengths = np.load(npy_path)
        all_lengths.append(lengths)

    max_lengths = np.max(np.stack(all_lengths), axis=0)
    print("Mean max_length:", np.mean(max_lengths))
    np.save(os.path.join(scratch_dir, "max.npy"), max_lengths)
