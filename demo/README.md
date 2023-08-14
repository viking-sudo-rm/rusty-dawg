# Rusty-DAWG demos

For a Jupyter notebook demonstrating function calls to the Python API, see `demo.ipynb`.

To run a gradio app that runs the DAWG for a user input query and visualizes the results, run:

```shell
DAWG_PATH=[path-to-dawg-file] gradio app.py
```

If a tokenizer other than `gpt-2` was used to construct the DAWG, you should also set the name of the tokenizer you want to use:

```shell
DAWG_PATH=[path-to-dawg-file] TOKENIZER=[tokenizer] gradio app.py
```
