"""Python script to fill counts in a CDAWG where this has not been done.

You NORMALLY don't need to run this, but it is useful in case you accidentally build a CDAWG without
calling fill_counts. This was the case in a legacy version of the codebase when building in RAM and
saving to disk, which is why I wrote this script.
"""

import argparse
import os

from rusty_dawg import DiskCdawg

parser = argparse.ArgumentParser()
parser.add_argument("path", type=str)
args = parser.parse_args()

train_path = os.path.join(args.path, "train.vec")
cdawg_path = os.path.join(args.path, "cdawg")
cdawg = DiskCdawg.load(train_path, cdawg_path)
print("Starting to fill counts...")
cdawg.fill_counts_ram()
print("Successfully filled counts in RAM!")