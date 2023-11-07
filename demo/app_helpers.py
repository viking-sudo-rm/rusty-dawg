import spacy
from spacy import displacy
from spacy.tokens import Span
from rusty_dawg import Dawg, DiskDawg, PyDawg
from transformers import GPT2Tokenizer
import pandas as pd
from typing import List
import os


# Get dawg path and tokenizer from environment variables. Can't pass them in as args
# because gradio reload mode doesn't play nicely with argparse.
dawg_path = os.getenv("DAWG_PATH")
tokenizer = os.getenv("TOKENIZER", "gpt2")

# Use the type of path to determine the Dawg format (Disk or RAM).
if dawg_path.endswith(".dawg"):
    DawgType = Dawg
elif os.path.isdir(dawg_path):
    DawgType = DiskDawg
elif not os.path.exists(dawg_path):
    raise ValueError(f"Path doesn't exist: {dawg_path}")
else:
    raise ValueError(f"Unknown DAWG format: {dawg_path}")

# Set globals.
DAWG = DawgType.load(dawg_path)
NLP = spacy.load("en_core_web_sm")
TOKENIZER = GPT2Tokenizer.from_pretrained(tokenizer)
PY_DAWG = PyDawg(DAWG, TOKENIZER)
COLORS = [
    "#a1c9f4",
    "#ffb482",
    "#8de5a1",
    "#ff9f9b",
    "#d0bbff",
    "#debb9b",
    "#fab0e4",
    "#cfcfcf",
    "#fffea3",
    "#b9f2f0",
]


def find_token_indices(doc: spacy.tokens.doc.Doc, substring: str):
    """
    Align the substrings we get from the GPT-2 tokenizer against the tokenization we get
    from Spacy, so that we can visualize matching text spans.
    """
    # TODO(davidw): This was mostly written by GPT-4. It's usually pretty good but some
    # edge cases are incorrect; should re-write if time.
    occurrences = []
    substring_start = doc.text.find(substring)
    substring_length = len(substring)

    while substring_start != -1:
        substring_end = substring_start + substring_length - 1
        start_token_idx = None
        end_token_idx = None

        for token in doc:
            if start_token_idx is None and token.idx >= substring_start:
                start_token_idx = token.i
            if end_token_idx is None and token.idx + len(token) - 1 >= substring_end:
                end_token_idx = token.i
                # Catch edge case where start token was never found.
                if start_token_idx is None:
                    start_token_idx = end_token_idx
                break

        occurrences.append((start_token_idx, end_token_idx))

        # Look for the next occurrence of the substring
        substring_start = doc.text.find(substring, substring_start + 1)

    return occurrences


def make_spacy_spans(doc: spacy.tokens.doc.Doc, matches: List):
    spans = []

    for match in matches:
        substring = match["text"]
        occurrences = find_token_indices(doc, substring)
        span_id = f"{match['id']}|{match['length']}|{match['count']}"

        for occurrence in occurrences:
            # NOTE: We add 1 because Spacy spans are exclusive on the right, but our
            # `occurrences` are inclusive.
            span = Span(doc, occurrence[0], occurrence[1] + 1, label=span_id)
            spans.append(span)

    return spans


def make_html(standardized_query: str, matches: List):
    doc = NLP(standardized_query)

    spans = make_spacy_spans(doc, matches)
    doc.spans["sc"] = spans

    uniq_labels = sorted(set(span.label_ for span in spans))
    # Repeat the colors enough times that we won't run out.
    color_wheel = COLORS * (len(uniq_labels) // len(COLORS) + 1)
    color_map = {label: color for label, color in zip(uniq_labels, color_wheel)}

    options = {"colors": color_map}
    html = displacy.render(doc, style="span", page=False, options=options)

    # Don't make the font size too small for the span annotations.
    html = html.replace("font-size: 0.6em; ", " ")

    return html


def run_query(query: str, min_tokens: int):
    matching_substrings = PY_DAWG.get_matching_substrings(query)

    # Only keep the substrings that are long enough.
    id_counter = 0
    matches = []
    for match in matching_substrings["matches"]:
        if len(match["tokens"]) >= min_tokens:
            match["length"] = len(match["tokens"])
            match["id"] = id_counter
            matches.append(match)
            id_counter += 1

    # Make DataFrame to display of all matching substrings.
    df = pd.DataFrame(
        [{k: entry[k] for k in ["id", "length", "count", "text"]} for entry in matches]
    )

    # Whitespace is removed from the `text` of the `matching substrings` when the
    # tokenizer `decodes` token sequences. To match the matching substrings, we also
    # remove whitespace from the full query.
    standardized_query = TOKENIZER.decode(matching_substrings["tokens"])
    html = make_html(standardized_query, matches)

    return html, df
