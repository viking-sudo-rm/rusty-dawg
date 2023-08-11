from app_helpers import run_query
import gradio as gr


wikitext_example = """In January 1844 , North Carolina Representative James Iver McKay ,
the chairman of the Committee on Ways and Means , solicited the views of Director
Patterson on the gold dollar
"""

mj_example = """Michael Jeffrey Jordan (born February 17, 1963), also known by his
initials MJ,[9] is an American former professional basketball player and businessman.
The official National Basketball Association (NBA) website states: "By acclamation,
Michael Jordan is the greatest basketball player of all time."
"""

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
