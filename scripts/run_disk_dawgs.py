import argparse
import subprocess
from tqdm import tqdm
import os
import json
from transformers import AutoTokenizer
import numpy as np

from rusty_dawg import Dawg, DiskDawg

COMMAND = """python3 scripts/run_disk_dawg.py \
    --dawg_dir={dawg_dir} \
    --data_path={data_path} \
    --no_print \
    --npy_path={npy_path} \
    --n_lines={n_lines} \
    --log_every={log_every} \
    --max_tokens={max_tokens}
"""

def build_dawgs_in_parallel(args, splits):
    for b in range(0, len(splits), args.batch_size):
        print("Starting batch", b, "...")
        procs = []
        batch_splits = splits[b: min(b + args.batch_size, len(splits))]
        for split in batch_splits:
            dawg_dir = os.path.join(args.dawgs_dir, split)
            npy_path = os.path.join(args.output_dir, "lengths_" + split + ".npy")
            command = COMMAND.format(dawg_dir=dawg_dir,
                                     data_path=args.data_path,
                                     npy_path=npy_path,
                                     n_lines=args.n_lines,
                                     log_every=args.log_every,
                                     max_tokens=args.max_tokens,)
            procs.append(subprocess.Popen(command, shell=True))
        for proc in procs:
            proc.wait()

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--dawgs_dir", type=str)
    parser.add_argument("--data_path", type=str)
    parser.add_argument("--output_dir", type=str)
    parser.add_argument("--n_lines", type=int, default=100000000)
    parser.add_argument("--log_every", type=int, default=2000)
    parser.add_argument("--batch_size", type=int, default=10, help="# DAWGs to run in parallel")
    parser.add_argument("--ignore_exists", action="store_true")
    parser.add_argument("--max_tokens", type=int, default=1500)
    parser.add_argument("--skip_dawgs", action="store_true")
    args = parser.parse_args()

    os.makedirs(args.output_dir, exist_ok=True)
    splits = os.listdir(args.dawgs_dir)

    if args.ignore_exists:
        splits = [split for split in splits if not os.path.exists(os.path.join(args.output_dir, "lengths_" + split + ".npy"))]
        if input(f"Confirm run {len(splits)} splits that do not exist yet? [y]") != "y":
            exit()

    if not args.skip_dawgs:
        build_dawgs_in_parallel(args, splits)

    print("=" * 50)
    print("All processes done!")

    print("Saving token info...")
    tokenizer = AutoTokenizer.from_pretrained("gpt2")
    all_tokens = []
    all_doc_ids = []
    metadata = {"documents": []}
    with open(args.data_path) as fh:
        for doc_id in range(args.n_lines):
            line = fh.readline()
            if not line:
                continue
            blob = json.loads(line)
            tokens = tokenizer(blob["text"]).input_ids
            tokens = tokens[:args.max_tokens]

            if "meta" in blob:
                meta = blob["meta"]
                meta["id"] = doc_id
            else:
                meta = {"id": doc_id}

            meta["text"] = blob["text"]

            all_tokens.extend(tokens)
            all_doc_ids.extend(doc_id for _ in tokens)
            metadata["documents"].append(meta)

    print("Saving, tokens, document IDs, and metadata...")
    np.save(os.path.join(args.output_dir, "tokens.npy"), np.array(all_tokens))
    np.save(os.path.join(args.output_dir, "doc_ids.npy"), np.array(all_doc_ids))
    with open(os.path.join(args.output_dir, "metadata.json"), "w") as fh:
        json.dump(metadata, fh)

    print("Maximizing lengths...")
    all_lengths = []
    for split in os.listdir(args.dawgs_dir):
        npy_path = os.path.join(args.output_dir, "lengths_" + split + ".npy")
        lengths = np.load(npy_path)
        all_lengths.append(lengths)

    max_lengths = np.max(np.stack(all_lengths), axis=0)
    print("Mean max_length:", np.mean(max_lengths))
    np.save(os.path.join(args.output_dir, "lengths.npy"), max_lengths)

    print("Retokenizing to Pythia...")
    from utils.retokenizer import Retokenizer
    from utils.dawg_results import DawgResults
    res = DawgResults(np.array(all_tokens), max_lengths, np.array(all_doc_ids), metadata)
    retokenizer = Retokenizer()
    pythia_lengths = retokenizer.get_retokenized_lengths_by_doc_id(res)
    with open(os.path.join(args.output_dir, "pythia_lengths.json"), "w") as fh:
        json.dump(pythia_lengths, fh)
