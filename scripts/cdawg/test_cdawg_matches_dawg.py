from argparse import ArgumentParser
from tqdm import tqdm
from transformers import GPT2TokenizerFast
import numpy as np

from rusty_dawg import Dawg, Cdawg

def parse_args():
    parser = ArgumentParser()
    parser.add_argument("--train", type=str, default="data/wikitext-2-raw/wiki.train.raw")
    parser.add_argument("--valid", type=str, default="data/wikitext-2-raw/wiki.valid.raw")
    parser.add_argument("--n_valid", type=int, default=None)
    return parser.parse_args()

def get_tokens(tokenizer, path):
    all_tokens = []
    for line in tqdm(open(path), desc=f"Open {path}..."):
        tokens = tokenizer(line)["input_ids"]
        all_tokens.extend(tokens)
    return all_tokens

if __name__ == "__main__":
    args = parse_args()
    tokenizer = GPT2TokenizerFast.from_pretrained("gpt2")
    train = get_tokens(tokenizer, args.train)
    valid = get_tokens(tokenizer, args.valid)
    if args.n_valid is not None:
        valid = valid[:args.n_valid]

    print("Building DAWG...")
    dawg = Dawg()
    dawg.build(train)

    print("Building CDAWG...")
    cdawg = Cdawg(train)
    cdawg.build()

    ds, length = (dawg.get_initial(), 0)
    cs = cdawg.get_initial()

    dlengths = []
    clengths = []

    for idx, token in enumerate(valid):
        ds, length = dawg.transition_and_count(ds, token, length)
        cs = cdawg.transition_and_count(cs, token)
        dlengths.append(length)
        clengths.append(cs.get_length())

    mismatched, = np.nonzero(np.array(clengths) != np.array(dlengths))
    print("Mismatched indices:", mismatched)

    import matplotlib.pyplot as plt
    plt.figure()
    plt.plot(clengths, label="CDAWG")
    plt.plot(dlengths, label="DAWG")
    plt.xlabel("validation token index")
    plt.ylabel("suffix context length")
    plt.tight_layout()
    plt.legend()
    plt.show()
