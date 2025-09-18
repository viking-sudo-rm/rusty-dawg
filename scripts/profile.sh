# Examples of how to profile with flame graph.

CORPUS=wikitext-2000

flamegraph --root -o flame.svg -- ./target/release/rusty-dawg --train-path "data/$CORPUS/wiki.train.raw" --test-path "data/$CORPUS/wiki.valid.raw" --save-path "/tmp/dawg" --results-path "" --n-eval 0 --tokenize