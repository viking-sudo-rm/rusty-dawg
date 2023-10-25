DATA_PATH="${DATA:-data}"

# More GPT-2 tokens compared to 

./target/release/rusty-dawg \
    --train-path "$DATA_PATH/$1/wiki.train.raw" \
    --n-tokens 2417786 \
    --nodes-ratio 1.25 \
    --edges-ratio 2.20 \
    --disk-path "/tmp/$1-dawg" \
    --tokenizer gpt2 \
    --utype u16

    # --max-state-length
