from .rusty_dawg import *  # This Python library wraps the built Rust library.
from transformers.tokenization_utils import PreTrainedTokenizer
from typing import Dict


class PyDawg:
    """
    Provides a Python API to query a DAWG.
    """

    def __init__(self, dawg: Dawg, tokenizer: PreTrainedTokenizer):
        """
        Construct DAWG API wrapper.
        """
        self.dawg = dawg
        self.tokenizer = tokenizer

    def get_suffix_context(self, query: str):
        """
        Given a `query` string, compute the suffix context for each prefix.
        """
        tokens = self.tokenizer.encode(query)
        state = self.dawg.get_initial()
        length = 0
        lengths = []
        counts = []
        for token in tokens:
            # TODO check on correctness.
            state, length = self.dawg.transition_and_count(state, token, length)
            lengths.append(length)
            count = self.dawg.get_count(state)
            counts.append(count)

        res = {"tokens": tokens, "suffix_contexts": lengths, "context_counts": counts}
        return res

    def get_matching_substrings(
        self, query: str, min_length: int = 1, remove_redundant: bool = True
    ):
        """
        Get list of all substrings of `query` that exist are present in the corpus.
        """
        sc = self.get_suffix_context(query)

        # Get all longest matching substrings and counts for each.
        matching_substrings = []
        for i in range(len(sc["tokens"])):
            token_prefix = sc["tokens"][: i + 1]
            suffix_context = sc["suffix_contexts"][i]

            # If there's no match for this token, skip.
            if suffix_context == 0:
                continue

            count_loop = sc["context_counts"][i]
            longest_match = tuple(token_prefix[-suffix_context:])
            text = self.tokenizer.decode(longest_match)
            token_indices = tuple(range(i - suffix_context + 1, i + 1))

            to_append = {
                "tokens": longest_match,
                "token_indices": token_indices,
                "text": text,
                "count": count_loop,
            }
            matching_substrings.append(to_append)

        if remove_redundant:
            matching_substrings = self.remove_redundant_substrings(matching_substrings)

        # Only keep substrings above the min length.
        matching_substrings = [
            entry for entry in matching_substrings if len(entry["tokens"]) >= min_length
        ]

        res = {"query": query, "tokens": sc["tokens"], "matches": matching_substrings}

        return res

    @staticmethod
    def remove_redundant_substrings(matching_substrings: Dict):
        """
        Remove substrings that only occur in train data as part of a longer substring.
        """
        res = []

        successor = matching_substrings[-1]
        res.append(successor)

        for entry in matching_substrings[:-1][::-1]:
            prefix_of_successor = (
                entry["token_indices"]
                == successor["token_indices"][: len(entry["token_indices"])]
            )
            same_count_as_successor = entry["count"] == successor["count"]
            if (not prefix_of_successor) or (not same_count_as_successor):
                res.append(entry)
                successor = entry

        # Convert back to original order.
        return res[::-1]
