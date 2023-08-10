from py_rusty_dawg import Dawg


class PyDawg:
    """
    Provides a Python API to conveniently query a DAWG.
    """

    def __init__(self, dawg_path, tokenizer):
        """

        """
        self.dawg = Dawg.load(dawg_path)
        self.tokenizer = tokenizer

    def get_suffix_context(self, query):
        tokens = self.tokenizer.encode(query)
        state = self.dawg.get_initial()
        length = 0
        lengths = []
        counts = []
        for token in tokens:
            state, length = self.dawg.transition_and_count(state, token, length)
            lengths.append(length)
            count = self.dawg.get_count(state)
            counts.append(count)

        res = {"tokens": tokens, "suffix_contexts": lengths, "context_counts": counts}
        return res

    def get_matching_substrings(self, query):
        """
        Get list of all substrings of `query` that exist are present in the corpus.
        """
        # TODO in progress. Need to remove redundant substrings.

        sc = self.get_suffix_context(query)

        # Get all longest matching substrings and counts for each.
        matching_substrings = {}
        for i in range(len(sc["tokens"])):
            token_prefix = sc["tokens"][: i + 1]
            suffix_context = sc["suffix_contexts"][i]
            # If there's no match for this token, skip.
            if suffix_context == 0:
                continue

            count_loop = sc["context_counts"][i]
            longest_match = tuple(token_prefix[-suffix_context:])
            if longest_match in matching_substrings:
                # If we've already seen this string before, confirm the count matches.
                if count_loop != matching_substrings[longest_match]:
                    raise ValueError("Count mismatch!")
            # Otherwise, add to dict.
            else:
                matching_substrings[longest_match] = count_loop

        # Convert to list of dicts.
        res = []
        for substring, count in matching_substrings.items():
            entry = {
                "tokens": substring,
                "count": count,
                "text": self.tokenizer.decode(substring),
            }
            res.append(entry)

        return res
