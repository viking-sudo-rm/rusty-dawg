from tqdm import tqdm
from transformers import AutoTokenizer

from .dawg_results import DawgResults

class Retokenizer:

  def __init__(self):
    self.gpt2_tokenizer = AutoTokenizer.from_pretrained("gpt2")
    self._tokenizers = {}

  def get_tokenizer(self, model):
    if model not in self._tokenizers:
      self._tokenizers[model] = AutoTokenizer.from_pretrained(model)
    return self._tokenizers[model]

  def get_retokenized_lengths_by_doc_id(self, res: DawgResults) -> list:
    """Return list of retokenized lengths"""
    lengths_by_doc_id = res.get_lengths_by_doc_id()
    retokenized_lengths_by_doc_id = {}
    for doc_id, lengths in tqdm(lengths_by_doc_id.items(), desc="Retokenizing"):
      document = res.metadata["documents"][doc_id]
      if "model" in document:
        tokenizer = self.get_tokenizer(document["model"])
      else:
        tokenizer = self.get_tokenizer("EleutherAI/pythia-12b")

      token_spans = [res.tokens[idx + 1 - length: idx + 1] for idx, length in enumerate(lengths)]
      text_spans = [self.gpt2_tokenizer.decode(span, skip_special_tokens=True) for span in token_spans]
      retokenized_lengths = [len(tokenizer(span).input_ids) for span in text_spans]
      retokenized_lengths_by_doc_id[doc_id] = retokenized_lengths
    return retokenized_lengths_by_doc_id

