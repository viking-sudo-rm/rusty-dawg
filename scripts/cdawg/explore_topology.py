"""Traverse graph and record arities of nodes.

Example usage:
```shell
python scripts/cdawg/explore_topology.py \
    --save-path ~/Desktop/arities.json \
    --plot-path ~/Desktop/arities.pdf
```

Can add `--load-path` to use saved data.
"""

from argparse import ArgumentParser
import os
from tqdm import tqdm
import json
import numpy as np

from rusty_dawg import DiskCdawg

def parse_args():
    parser = ArgumentParser()
    parser.add_argument("--path", type=str, default="/tmp/wikitext-2-raw", 
                        help="CDAWG path, defaults to path used by scripts/cdawg/run_local.sh")
    parser.add_argument("--load-path", type=str, default=None)
    parser.add_argument("--save-path", type=str, default=None)
    parser.add_argument("--plot-path", type=str, default=None)
    parser.add_argument("--size", type=int, default=None)
    return parser.parse_args()

def get_arities(args) -> list[int]:
    tokens_path = os.path.join(args.path, "train.vec")
    cdawg_path = os.path.join(args.path, "cdawg")
    cdawg = DiskCdawg.load(tokens_path, cdawg_path)
    if args.size is None:
        print("CDAWG size not provided; computing...")
    size = args.size if args.size is not None else cdawg.node_count()
    print("Starting to traverse arities...")
    return cdawg.traverse_arities(size)

def main(args):
    if not args.load_path:
        arities = get_arities(args)
        if args.save_path is not None:
            with open(args.save_path, "w") as fh:
                json.dump(arities, fh)
    else:
        with open(args.load_path) as fh:
            arities = json.load(fh)
    
    arities = np.array(arities)
    print("=== Arity Stats ===")
    for p in [50, 75, 99, 99.9, 99.99]:
        print(f"  {p}%: {np.percentile(arities, p):.2f}")
    print("  max:", np.max(arities))

    if args.plot_path is not None:
        import matplotlib.pyplot as plt
        import seaborn as sns
        plt.figure()
        sns.set_style()
        sns.set_theme(style="darkgrid")
        sns.histplot(np.log10(arities), bins=20, kde=True)
        # plt.yscale("log")
        plt.xlabel("state arity")
        xticks = [int(x) for x in [1e1, 1e2, 1e3, 1e4, 5e4]]
        plt.xticks(np.log10(xticks), xticks)
        plt.ylim(1, 5e5)
        sns.despine()
        plt.savefig(args.plot_path)

if __name__ == '__main__':
    main(parse_args())