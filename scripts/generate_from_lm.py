"""Generate text from a Huggingface checkpoint

Reference for model generation: https://huggingface.co/blog/how-to-generate
Reference for Pythia checkpoints: https://huggingface.co/EleutherAI/pythia-6.9b
"""

import tqdm
import json
import torch
from argparse import ArgumentParser
import os

from transformers import AutoTokenizer, GPTNeoXForCausalLM
from transformers import set_seed

def get_params_grid(args):
    all_params = {}
    all_params["sample"] = dict(do_sample=True)  # By default, this is the only parameter that gets used.
    for b in args.beam:
        all_params[f"beam={b}-norepeat=2"] = dict(num_beams=b, no_repeat_ngram_size=2)
        all_params[f"beam={b}"] = dict(num_beams=b)
    for k in args.top_k:
        all_params[f"top_k={k}"] = dict(do_sample=True, top_k=k)
    for p in args.top_p:
        all_params[f"top_p={p}"] = dict(do_sample=True, top_p=p)
    for temp in args.temperature:
        all_params[f"temp={temp}"] = dict(do_sample=True, temperature=temp)
    return all_params

@torch.no_grad()
def iterate_generate(tokenizer, model, title, params: dict, full_length: int, context_length: int = 512, stride: int = 512, seed: int = 42, n_return: int = 1, device="cuda"):
    set_seed(seed)
    input_ids = tokenizer.encode("The", return_tensors="pt")
    pbar = tqdm.tqdm(total=full_length, desc=title)
    pbar.update(input_ids.size(1))
    while input_ids.size(1) < full_length:
        context = input_ids[:, -context_length:].to(device)
        n_return = n_return if context.size(0) == 1 else 1
        output_ids = model.generate(context,
                                    max_new_tokens=stride,
                                    pad_token_id=50256,
                                    num_return_sequences=n_return,
                                    **params)
        if input_ids.size(0) == output_ids.size(0):
            input_ids = torch.cat([input_ids[:, :-context_length], output_ids.cpu()], dim=1)
        else: # Only reachable on first batch.
            input_ids = output_ids.cpu()
        pbar.update(input_ids.size(1) - pbar.n)
    pbar.close()
    return input_ids

def save_jsonl(tokenizer, all_input_ids, name, args):
    # Only do multiple seeds for nondeterministic decoding strategies.
    # seeds = range(args.seed, args.seed + args.n_seeds) if "do_sample" in params else [args.seed]
    with open(args.save_path, "w") as fh:
        for seed, input_ids in enumerate(all_input_ids):
            text = tokenizer.decode(input_ids, skip_special_tokens=True)
            blob = {
                "text": text,
                "meta": {
                    "model": args.model,
                    "decoding": name,
                    "seed": seed,
                }
            }
            fh.write(json.dumps(blob))
            fh.write("\n")

def parse_args():
    parser = ArgumentParser()
    parser.add_argument("model", type=str)
    parser.add_argument("save_path", type=str)
    parser.add_argument("--n_tokens", type=int, default=10000)
    parser.add_argument("--seed", type=int, default=42)
    parser.add_argument("--n_seeds", type=int, default=10)
    parser.add_argument("--batch_size", type=int, default=10)
    # Arguments for grid search
    parser.add_argument("-t", "--temperature", type=int, nargs="+", default=[],
                        help="List of temperatures to decode with")
    parser.add_argument("-k", "--top_k", type=int, nargs="+", default=[],
                        help="top-k sampling parameter list")
    parser.add_argument("-p", "--top_p", type=float, nargs="+", default=[],
                        help="top-p/nucleus sampling parameter list")
    parser.add_argument("-b", "--beam", type=int, nargs="+", default=[],
                        help="Beam sizes for argmax decoding")
    return parser.parse_args()

def main(args):
    args.device = "cuda" if torch.cuda.is_available() else "cpu"
    args.batch_size = min(args.batch_size, args.n_seeds)
    tokenizer = AutoTokenizer.from_pretrained(args.model, padding_side="left")
    model = GPTNeoXForCausalLM.from_pretrained(args.model)
    model.cuda()

    all_params = get_params_grid(args)
    for name, params in all_params.items():
        all_input_ids = None
        while all_input_ids is None or all_input_ids.size(0) < args.n_seeds:
            n_return = min(args.batch_size, args.n_seeds - (0 if all_input_ids is None else all_input_ids.size(0)))
            input_ids = iterate_generate(tokenizer, model, name, params, full_length=args.n_tokens, seed=args.seed, n_return=n_return, device=args.device).squeeze()
            if all_input_ids is None:
                all_input_ids = input_ids
            else:
                all_input_ids = torch.concatenate([all_input_ids, input_ids], dim=0)
        save_jsonl(tokenizer, all_input_ids, name, args)

if __name__ == "__main__":
    main(parse_args())
