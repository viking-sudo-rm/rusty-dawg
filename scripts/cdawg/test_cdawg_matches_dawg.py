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
    all_tokens.append(Cdawg.EOS)
    return all_tokens

def get_count(tokens, ngram):
    """Count actual # of occurrences of ngram in tokens in linear time"""
    count = 0
    for idx in range(0, len(tokens) - len(ngram)):
        data_ngram = tokens[idx: idx + len(ngram)]
        if data_ngram == ngram:
            count += 1
    return count

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

    cdawg = Cdawg(train)
    print("Building CDAWG...")
    cdawg.build()
    print("Filling CDAWG counts...")
    cdawg.fill_counts()

    ds, length = (dawg.get_initial(), 0)
    cs = cdawg.get_initial()

    dlengths = []
    clengths = []

    dcounts = []
    ccounts = []

    for idx, token in enumerate(valid):
        ds, length = dawg.transition_and_count(ds, token, length)
        cs = cdawg.transition_and_count(cs, token)
        dlengths.append(length)
        clengths.append(cs.get_length())
        dcounts.append(dawg.get_count(ds))
        ccounts.append(cdawg.get_suffix_count(cs))

    mismatched, = np.nonzero(np.array(clengths) != np.array(dlengths))
    print("Mismatched length indices:", mismatched)

    mismatched, = np.nonzero(np.array(ccounts) != np.array(dcounts))
    print("Mismatched count indices:", mismatched)
    print("CDAWG counts:", ccounts[:20])
    print("DAWG counts:", dcounts[:20])
    ngrams = [valid[idx + 1 - length: idx + 1] for idx, length in enumerate(clengths[:20])]
    actual_counts = [get_count(train, ngram) for ngram in ngrams]
    print("Actual counts:", actual_counts)

    source = cdawg.get_source()
    print("count(source) =", cdawg.get_count(source), "#(tokens) =", len(train))

    import matplotlib.pyplot as plt
    plt.figure()
    plt.plot(clengths, label="CDAWG")
    plt.plot(dlengths, label="DAWG")
    plt.xlabel("validation token index")
    plt.ylabel("suffix context length")
    plt.tight_layout()
    plt.legend()
    plt.show()

    plt.figure()
    plt.plot(ccounts, label="CDAWG")
    # plt.plot(dcounts, label="DAWG")
    plt.xlabel("validation token index")
    plt.ylabel("suffix context frequency")
    plt.tight_layout()
    plt.legend()
    plt.show()
