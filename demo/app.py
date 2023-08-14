"""
Usage example:
DAWG_PATH=[path-to-dawg-file] gradio app.py
"""

from app_helpers import run_query
import app_info
import gradio as gr


# Launch the app.

with gr.Blocks(css=app_info.span_css) as demo:
    title = gr.Markdown(value=app_info.description)
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
            examples = gr.Examples(examples=app_info.examples, inputs=query)
    with gr.Row(variant="panel"):
        html = gr.HTML()

    df = gr.DataFrame(headers=["id", "length", "count", "text"])

    button.click(run_query, inputs=[query, slider], outputs=[html, df])


demo.launch(share=True)
