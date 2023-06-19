// Implementation of Suffix DFA in Rust.
// 
// See here for Graph info:
// https://docs.rs/petgraph/latest/petgraph/graph/struct.Graph.html
// 
// See here for Suffix Automaton algorithm in Python:
// https://github.com/viking-sudo-rm/knn-transformers/blob/master/src/suffix_dfa_builder.py
// 

extern crate petgraph;
extern crate kdam;
extern crate substring;
extern crate serde;
extern crate serde_json;
extern crate bitvec;
extern crate bincode;
extern crate tempfile;

mod dawg;
mod weight;
mod stat_utils;
mod token_index;
mod graph;
mod evaluator;
mod lms;

use lms::LM;
use lms::kn_lm::KNLM;
use lms::induction_lm::InductionLM;

use std::mem::size_of;
use std::fs;
use std::env;
use bincode::{serialize_into, deserialize_from};

use kdam::tqdm;

use stat_utils::*;
use dawg::Dawg;
use weight::BasicWeight;
use token_index::TokenIndex;
use evaluator::Evaluator;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("sizeof(edge): {}B", size_of::<E>());
    println!("sizeof(node): {}B", size_of::<BasicWeight>());

    let args: Vec<String> = env::args().collect();
    let train_path = &args[1];
    let test_path = &args[2];
    let save_path = &args[3];
    let results_path = &args[4];

    let train_raw: String = fs::read_to_string(train_path).expect("Error loading train");
    let test_raw: String = fs::read_to_string(test_path).expect("Error loading test");

    // Load at word level.
    type E = usize;
    let mut index = TokenIndex::new();
    let train: Vec<usize> = train_raw.split_whitespace().map(|x| index.add(x)).collect();
    let mut test: Vec<usize> = test_raw.split_whitespace().map(|x| index.add(x)).collect();

    // We are currently ignoring the probability of <eos>, very negligible
    // let eos = index.index("<eos>");
    // train.push(eos);
    // test.push(eos);

    let old_test_len = test.len();
    let n_test = 10000;
    test = (&test[0..n_test]).to_vec();
    let eval_threshold = train.len() / 20;

    println!("#(train): {}", train.len());
    println!("#(test): {}/{}", test.len(), old_test_len);
    println!("#(vocab): {}", index.count);

    let mut lms: Vec<Box<dyn LM>> = Vec::new();
    for delta in vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9].iter() {
        let maxgram = KNLM::new(format!("maxgram_kn-{}", delta), *delta, -1);
        lms.push(Box::new(maxgram));
        let ngram = KNLM::new(format!("ngram_kn-{}", delta), *delta, 4);
        lms.push(Box::new(ngram));
    }
    for delta in vec![0.5, 0.55, 0.6, 0.65, 0.7, 0.75, 0.8, 0.85, 0.9, 0.95].iter() {
        let induct_backoff = KNLM::new("ngram_kn-0.7".to_string(), 0.7, 4);
        let induct = InductionLM::new(format!("induct-{}", delta), Box::new(induct_backoff), *delta);
        lms.push(Box::new(induct))
    }
    let mut evaluator = Evaluator::new(&mut lms, &test);

    let mut dawg: Dawg<E> = Dawg::new();
    let mut last = dawg.get_initial();
    for (idx, token) in tqdm!(train.iter()).enumerate() {
        last = dawg.extend(*token, last);
        if idx % eval_threshold == 0 {
            let good_turing = good_turing_estimate(&dawg, train.len());        
            evaluator.evaluate(&dawg, idx, good_turing);
            checkpoint(&dawg, &evaluator, results_path, save_path)?;
        }
    }
    checkpoint(&dawg, &evaluator, results_path, save_path)?;
    println!("Completed!");
    println!("  Node count: {}", dawg.node_count());
    println!("  Edge count: {}", dawg.edge_count());

    Ok(())
}

fn checkpoint(dawg: &Dawg<usize>, evaluator: &Evaluator<usize>, results_path: &str, save_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    evaluator.to_json(results_path)?;
    println!("Successfully saved results to {}!", results_path);

    let mut save_file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(save_path)?;
    serialize_into(&save_file, &dawg)?;
    println!("Successfully saved DAWG to {}!", save_path);

    Ok(())
}