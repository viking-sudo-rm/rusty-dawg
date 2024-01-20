from typing import NamedTuple
from collections import defaultdict
import os
import json
import numpy as np


class DawgResults(NamedTuple):
  tokens: np.ndarray
  lengths: np.ndarray
  doc_ids: np.ndarray
  metadata: dict

  @classmethod
  def load(cls, dir_path):
    tokens = np.load(os.path.join(dir_path, "tokens.npy"))
    lengths = np.load(os.path.join(dir_path, "lengths.npy"))
    doc_ids = np.load(os.path.join(dir_path, "doc_ids.npy"))
    with open(os.path.join(dir_path, "metadata.json")) as fh:
      metadata = json.load(fh)
    return cls(tokens, lengths, doc_ids, metadata)

  def trim(self, n_tokens: int) -> "DawgResults":
    return DawgResults(
        tokens=self.tokens[:n_tokens],
        lengths=self.lengths[:n_tokens],
        doc_ids=self.doc_ids[:n_tokens],
        metadata=self.metadata,
    )

  def get_lengths_by_doc_id(self):
    lengths_by_doc_id = defaultdict(list)
    for doc_id, length in zip(self.doc_ids, self.lengths):
      lengths_by_doc_id[int(doc_id)].append(length)
    return lengths_by_doc_id

