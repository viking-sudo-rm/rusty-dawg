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
    --n_lines={n_lines}
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
         npy_path = os.path.join(scratch_dir, "lengths_" + split + ".npy")
         command = COMMAND.format(dawg_dir=dawg_dir, data_path=args.data_path, npy_path=npy_path, n_lines=args.n_lines)
         procs.append(subprocess.Popen(command, shell=True))

    for proc in procs:
        proc.wait()

    print("=" * 50)
    print("All processes done!")

    print("Saving token info...")
    tokenizer = AutoTokenizer.from_pretrained("gpt2")
    all_tokens = []
    all_doc_ids = []
    metadata = {"documents": []}
    with open(args.data_path) as fh:
        for doc_id in range(args.n_lines):
            blob = json.loads(fh.readline())
            tokens = tokenizer(blob["text"]).input_ids
            if "meta" in blob and "pile_set_name" in blob["meta"]:
                split = blob["meta"]["pile_set_name"]
            else:
                split = None
            all_tokens.extend(tokens)
            all_doc_ids.extend(doc_id for _ in tokens)
            metadata["documents"].append({"id": doc_id, "split": split})

    print("Saving, tokens, document IDs, and metadata...")
    np.save(os.path.join(scratch_dir, "tokens.npy"), np.array(all_tokens))
    np.save(os.path.join(scratch_dir, "doc_ids.npy"), np.array(all_doc_ids))
    with open(os.path.join(scratch_dir, "metadata.json"), "w") as fh:
        json.dump(metadata, fh)

    print("Maximizing lengths...")
    all_lengths = []
    for split in os.listdir(args.dawgs_dir):
        npy_path = os.path.join(scratch_dir, "lengths_" + split + ".npy")
        lengths = np.load(npy_path)
        all_lengths.append(lengths)

    max_lengths = np.max(np.stack(all_lengths), axis=0)
    print("Mean max_length:", np.mean(max_lengths))
    np.save(os.path.join(scratch_dir, "lengths.npy"), max_lengths)
