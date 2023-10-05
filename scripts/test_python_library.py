from transformers import AutoTokenizer

from rusty_dawg import Dawg, DiskDawg

def test_new_ram():
    dawg = Dawg()
    dawg.build([21, 34, 32])

def test_load_disk():
    path = "/tmp/wikitext-2-raw-dawg"
    tokenizer = AutoTokenizer.from_pretrained("gpt2")
    dawg = DiskDawg.load(path)

    tokens = tokenizer("the cat saw the world in the bottle").input_ids
    state = dawg.get_initial()
    length = 0
    lengths = []
    for token in tokens:
        state, length = dawg.transition_and_count(state, token, length)
        lengths.append(length)
    
    print(tokenizer.convert_ids_to_tokens(tokens))
    print(lengths)

if __name__ == "__main__":
    test_new_ram()
    test_load_disk()
