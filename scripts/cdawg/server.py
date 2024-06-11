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
        all_tokens = blob["tokens"]
    elif "text" in blob:
        texts = blob["text"]
        all_tokens = [tokenizer.encode(text) for text in texts]
    else:
        return jsonify({"error": "request must contain 'tokens' or 'text' key"})
    
    return_entropies = blob.get("return_entropies", False)
    return_next_tokens = blob.get("return_next_tokens", 0)
    
    # Need to do it this way because DiskCdawg is unsendable
    cdawg = DiskCdawg.load(train_path, cdawg_path)

    all_lengths = []
    all_counts = []
    all_entropies = []
    all_next_tokens = []
    for tokens in all_tokens:
        cs = cdawg.get_initial()
        lengths = []
        counts = []
        entropies = []
        next_tokens = []
        for token in tokens:
            cs = cdawg.transition_and_count(cs, token)
            lengths.append(cs.get_length())
            counts.append(cdawg.get_suffix_count(cs))
            if return_entropies:
                entropies.append(cdawg.get_entropy(cs))
            if return_next_tokens == -1:
                next_tokens.append(cdawg.get_next_tokens(cs))
            elif return_next_tokens > 0:
                next_tokens.append(cdawg.get_next_tokens(cs)[:return_next_tokens])
        all_lengths.append(lengths)
        all_counts.append(counts)
        if return_entropies:
            all_entropies.append(entropies)
        if return_next_tokens != 0:
            all_next_tokens.append(next_tokens)

    results = {
        "tokens": all_tokens,
        "lengths": all_lengths,
        "counts": all_counts,
    }
    if return_entropies:
        results["entropies"] = all_entropies
    if return_next_tokens != 0:
        results["next_tokens"] = all_next_tokens
    return jsonify(results)

if __name__ == "__main__":
    app.run(host="0.0.0.0", port=args.port, debug=True)