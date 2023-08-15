# Description, examples, and presentation.

description = """# Rusty Dawg Visualizer

Rusty DAWG enables very fast n-gram search over a large corpus, such as those used to
train LLMs. Given a query string as input, Rusty DAWG will search the for spans that
appear in both the corpus and the query string. It will render the matching spans,
indicating how often each one appears in the training data. Each span will be annotated
as `{span id}|{span length in tokens}|{span count in corpus}`."""

wikitext_example = """In January 1844 , North Carolina Representative James Iver McKay ,
the chairman of the Committee on Ways and Means , solicited the views of Director
Patterson on the gold dollar
"""

mj_example = """Michael Jeffrey Jordan (born February 17, 1963), also known by his
initials MJ,[9] is an American former professional basketball player and businessman.
The official National Basketball Association (NBA) website states: "By acclamation,
Michael Jordan is the greatest basketball player of all time."
"""

examples = [example.replace("\n", " ") for example in [wikitext_example, mj_example]]
span_css = ".spans { font-size: 16px; }"
