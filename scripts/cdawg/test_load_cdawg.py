"""
Example usage:
RESULT=/Users/willm/Desktop/run-local
python scripts/cdawg/test_load_cdawg.py $RESULT/train.vec $RESULT/cdawg
"""

from argparse import ArgumentParser
from transformers import AutoTokenizer

from rusty_dawg import DiskCdawg

TEXT = "hello to the best of friends"

def parse_args():
    args = ArgumentParser()
    args.add_argument("tokens_path", type=str)
    args.add_argument("cdawg_path", type=str)
    args.add_argument("--tokenizer", type=str, default="gpt2")
    return args.parse_args()

if __name__ == "__main__":
    args = parse_args()
    tokenizer = AutoTokenizer.from_pretrained(args.tokenizer)
    cdawg = DiskCdawg.load(args.tokens_path, args.cdawg_path)

    lengths = []
    cs = cdawg.get_initial()
    for token in tokenizer(TEXT).input_ids:
        cs = cdawg.transition_and_count(cs, token)
        lengths.append(cs.get_length())

    print("Got lengths", lengths)
