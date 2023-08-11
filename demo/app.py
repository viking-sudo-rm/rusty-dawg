from app_helpers import run_query
import gradio as gr


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

with gr.Blocks(css=span_css) as demo:
    title = gr.Markdown(value=description)
    with gr.Row(variant="panel"):
        with gr.Column(scale=1, min_width=400):
            query = gr.Textbox(label="Query")
            with gr.Row():
                with gr.Column(scale=1, min_width=50):
                    button = gr.Button(value="Run query", variant="primary")
                with gr.Column(scale=2, min_width=50):
                    slider = gr.Slider(
                        1, 5, value=4, step=1, label="Min n-gram display length"
                    )
        with gr.Column(scale=1, min_width=400):
            examples = gr.Examples(examples=examples, inputs=query)
    with gr.Row(variant="panel"):
        html = gr.HTML()

    df = gr.DataFrame(headers=["id", "length", "count", "text"])

    button.click(run_query, inputs=[query, slider], outputs=[html, df])


demo.launch(share=True)
