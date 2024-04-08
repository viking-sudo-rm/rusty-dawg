from argparse import ArgumentParser
from flask import Flask, jsonify, request
from transformers import AutoTokenizer
import os

from rusty_dawg import DiskCdawg

parser = ArgumentParser()
parser.add_argument("path", type=str)
parser.add_argument("--format", choices=["disk-cdawg"], default="disk-cdawg")  # unused for now
parser.add_argument("--tokenizer", default="EleutherAI/pythia-12b")
parser.add_argument("--port", type=int, default=80)
args = parser.parse_args()

app = Flask(__name__) 
tokenizer = AutoTokenizer.from_pretrained(args.tokenizer)
train_path = os.path.join(args.path, "train.vec")
cdawg_path = os.path.join(args.path, "cdawg")

@app.route("/api/cdawg", methods=["POST"])
def cdawg_inference():
    blob = request.json
    if "tokens" in blob:
        tokens = blob["tokens"]
    elif "text" in blob:
        text = blob["text"]
        tokens = tokenizer.encode(text)
    else:
        return jsonify({"error": "request must contain 'tokens' or 'text' key"})
    
    # Need to do it this way because DiskCdawg is unsendable
    cdawg = DiskCdawg.load(train_path, cdawg_path)

    lengths = []
    counts = []
    cs = cdawg.get_initial()
    for token in tokens:
        cs = cdawg.transition_and_count(cs, token)
        lengths.append(cs.get_length())
        counts.append(cdawg.get_suffix_count(cs))

    return jsonify({
        "tokens": tokens,
        "lengths": lengths,
        "counts": counts,
    })

if __name__ == "__main__":
    app.run(port=args.port)