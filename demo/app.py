import gradio as gr
import spacy
from spacy import displacy
from spacy.tokens import Span
from rusty_dawg import Dawg, PyDawg
from transformers import GPT2Tokenizer
import pandas as pd


# Globals

dawg_path = "../dawg/wikitext-2-raw.dawg"
DAWG = Dawg.load(dawg_path)
NLP = spacy.load("en_core_web_sm")
TOKENIZER = GPT2Tokenizer.from_pretrained("gpt2")
PY_DAWG = PyDawg(DAWG, TOKENIZER)
COLORS = [
    "#1f77b4",
    "#ff7f0e",
    "#2ca02c",
    "#d62728",
    "#9467bd",
    "#8c564b",
    "#e377c2",
    "#7f7f7f",
    "#bcbd22",
    "#17becf",
]


########################################

# Helper functions.


def find_token_indices(doc: spacy.tokens.doc.Doc, substring: str):
    """
    The goal here is to align the substrings we get from the GPT-2 tokenizer against the
    tokenization we get from Spacy, so that we can visualize matching text spans.
    """
    # TODO(davidw): This was mostly written by GPT-4. It's usually pretty good but some
    # edge cases are incorrect; should re-write if time.
    # TODO(davidw) also clean up the docs.
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


def make_spacy_spans(doc, matches):
    spans = []

    for i, match in enumerate(matches):
        substring = match["text"]
        occurrences = find_token_indices(doc, substring)
        span_id = f"{i}:{match['count']}"

        for occurrence in occurrences:
            # NOTE: We add 1 because Spacy spans are exclusive on the right, but our
            # `occurrences` are inclusive.
            span = Span(doc, occurrence[0], occurrence[1] + 1, label=span_id)
            spans.append(span)

    return spans


def make_html(standardized_query, matches):
    doc = NLP(standardized_query)

    spans = make_spacy_spans(doc, matches)
    doc.spans["sc"] = spans

    uniq_labels = sorted(set(span.label_ for span in spans))
    # Repeat the colors enough times that we won't run out.
    color_wheel = COLORS * (len(uniq_labels) // len(COLORS) + 1)
    color_map = {label: color for label, color in zip(uniq_labels, color_wheel)}

    options = {"colors": color_map}
    html = displacy.render(doc, style="span", page=True, options=options)
    html = (
        "<div style='max-width:100%; max-height:360px; overflow:auto'>"
        + html
        + "</div>"
    )
    return html


def run_query(query, min_tokens):
    matching_substrings = PY_DAWG.get_matching_substrings(query)

    # Only keep the substrings that are long enough.
    matches = [
        match
        for match in matching_substrings["matches"]
        if len(match["tokens"]) >= min_tokens
    ]

    # Make DataFrame to display of all matching substrings.
    for entry in matches:
        entry["length"] = len(entry["tokens"])
    df = pd.DataFrame(
        [{k: entry[k] for k in ["length", "count", "text"]} for entry in matches]
    )

    # Whitespace is removed from the `text` of the `matching substrings` when the
    # tokenizer `decodes` token sequences. To match the matching substrings, we also
    # remove whitespace from the full query.
    standardized_query = TOKENIZER.decode(matching_substrings["tokens"])
    html = make_html(standardized_query, matches)

    return html, df


########################################

# Build and run the app.

wikitext_example = """Senjō no Valkyria 3 : Unrecorded Chronicles ( Japanese : 戦場の
ヴァルキュリア3 , lit . Valkyria of the Battlefield 3 ) , commonly referred to as
Valkyria Chronicles III outside Japan , is a tactical role @-@ playing video game
developed by Sega and Media.Vision for the PlayStation Portable . Released in January
2011 in Japan , it is the third game in the Valkyria series . Employing the same fusion
of tactical and real @-@ time gameplay as its predecessors , the story runs parallel to
the first game and follows the " Nameless " , a penal military unit serving the nation
of Gallia during the Second Europan War who perform secret black operations and are
pitted against the Imperial unit " Calamaty Raven " ."""

mj_example = """Michael Jeffrey Jordan (born February 17, 1963), also known by his
initials MJ,[9] is an American former professional basketball player and businessman.
The official National Basketball Association (NBA) website states: "By acclamation,
Michael Jordan is the greatest basketball player of all time."[10] He played fifteen
seasons in the NBA, winning six NBA championships with the Chicago Bulls. He was
integral in popularizing the sport of basketball and the NBA around the world in the
1980s and 1990s,[11] becoming a global cultural icon.[12]"""

demo = gr.Interface(
    fn=run_query,
    inputs=[
        gr.Textbox(placeholder="Enter sentence here..."),
        gr.Slider(1, 5, value=4, step=1, label="Min n-gram display length"),
    ],
    outputs=["html", gr.DataFrame(headers=["length", "count", "text"])],
    examples=[
        [wikitext_example],
        [mj_example],
    ],
)

demo.launch(share=True)
