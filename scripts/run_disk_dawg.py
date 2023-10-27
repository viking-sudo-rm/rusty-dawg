import argparse
import numpy as np
from transformers import AutoTokenizer
import json

from rusty_dawg import Dawg, DiskDawg

def flatten(l):
    return [item for sublist in l for item in sublist]

def main(args):
    dawg = DiskDawg.load(args.dawg_dir)
    tokenizer = AutoTokenizer.from_pretrained(args.tokenizer)

    blobs = []
    with open(args.data_path) as fh:
        for _ in range(args.n_lines):
            blob = json.loads(fh.readline())
            blobs.append(blob)

    all_tokens = []
    all_lengths = []
    for blob in blobs:
        state = dawg.get_initial()
        length = 0
        tokens = tokenizer(blob["text"]).input_ids
        lengths = []

        for token in tokens:
            if not args.handle_exceptions:
                state, length = dawg.transition_and_count(state, token, length)
            else:
                try:
                    state, length = dawg.transition_and_count(state, token, length)
                except BaseException:
                    state = dawg.get_initial()
                    length = 0
            lengths.append(length)
        all_tokens.append(tokens)
        all_lengths.append(lengths)

    if not args.no_print:
        blob = {
            "tokens": all_tokens,
            "lengths": all_lengths,
        }
        print(json.dumps(blob))

    if args.npy_path is not None:
        flat_lengths = np.array(flatten(all_lengths))
        with open(args.npy_path, "wb") as fh:
            np.save(fh, flat_lengths)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--dawg_dir", type=str)
    parser.add_argument("--data_path", type=str)
    parser.add_argument("--handle_exceptions", action="store_true")
    parser.add_argument("--n_lines", type=int, default=1000)
    parser.add_argument("--tokenizer", type=str, default="gpt2")
    parser.add_argument("--npy_path", type=str, default=None)
    parser.add_argument("--no_print", action="store_true")
    main(parser.parse_args())
