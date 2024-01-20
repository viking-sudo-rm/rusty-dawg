import argparse
import numpy as np
from transformers import AutoTokenizer
import json
from functools import lru_cache

from rusty_dawg import Dawg, DiskDawg

CACHE_SIZE = 1000000

class CachedDawg:

    """Wrap a DAWG and implement LRU caching"""

    def __init__(self, dawg):
        self.dawg = dawg

    @classmethod
    def load(cls, path):
       return cls(DiskDawg.load(path))

    def get_initial(self):
        return self.dawg.get_initial()

    @lru_cache(maxsize=CACHE_SIZE)
    def transition_and_count(self, state, token, length):
        # FIXME: Somehow don't cache length here.
        return self.dawg.transition_and_count(state, token, length)

def flatten(l):
    return [item for sublist in l for item in sublist]

def main(args):
    dawg = CachedDawg.load(args.dawg_dir)
    tokenizer = AutoTokenizer.from_pretrained(args.tokenizer)

    blobs = []
    with open(args.data_path) as fh:
        for _ in range(args.n_lines):
            line = fh.readline()
            if not line:
                break
            blob = json.loads(line)
            blobs.append(blob)

    all_tokens = []
    all_lengths = []
    for i, blob in enumerate(blobs):
        if args.use_tokens:
            tokens = blob["tokens"]
        else:
            tokens = tokenizer(blob["text"]).input_ids
        tokens = tokens[:args.max_tokens]

        state = dawg.get_initial()
        length = 0
        lengths = []
        for j, token in enumerate(tokens):
            if not args.handle_exceptions:
                state, length = dawg.transition_and_count(state, token, length)
            else:
                try:
                    state, length = dawg.transition_and_count(state, token, length)
                except BaseException:
                    print("Error on", args.dawg_dir)
                    state = dawg.get_initial()
                    length = 0
            lengths.append(length)

            if args.log_every > 0 and j % args.log_every == 0:
                print(args.dawg_dir, "doc:", i, "/", len(blobs), "tok:", j, "/", len(tokens))
                print(dawg.transition_and_count.cache_info())

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
    parser.add_argument("--max_tokens", type=int, default=1500)
    parser.add_argument("--tokenizer", type=str, default="gpt2")
    parser.add_argument("--npy_path", type=str, default=None)
    parser.add_argument("--no_print", action="store_true")
    parser.add_argument("--log_every", type=int, default=0)
    parser.add_argument("--use_tokens", action="store_true")
    main(parser.parse_args())
