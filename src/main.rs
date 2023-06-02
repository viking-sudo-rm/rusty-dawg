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

mod dawg;
mod weight;
mod stat_utils;
mod token_index;
mod vec_graph;
mod evaluator;
mod lms;

use lms::LM;
use lms::kn_lm::KNLM;

// use std::cmp::max;
// use std::io::{self, Read};
use std::mem::size_of;
// use std::marker::Copy;
use std::fs;
// use std::fmt::Debug;
// use std::vec;
// use substring::Substring;

use kdam::tqdm;

use vec_graph::*;
use stat_utils::*;
use dawg::Dawg;
use weight::BasicWeight;
use token_index::TokenIndex;
use evaluator::Evaluator;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("sizeof(edge): {}B", size_of::<E>());
    println!("sizeof(node): {}B", size_of::<BasicWeight>());

    let train_path = "/Users/willm/Desktop/wikitext-2-raw/wiki.train.raw";
    let test_path = "/Users/willm/Desktop/wikitext-2-raw/wiki.valid.raw";
    let out_path = "/Users/willm/Desktop/wikitext2.json";
    // let train_path = "/Users/willm/Desktop/wikitext-103-raw/wiki.train.raw";
    // let test_path = "/Users/willm/Desktop/wikitext-103-raw/wiki.valid.raw";
    // let out_path = "/Users/willm/Desktop/wikitext103.json";

    let train_raw: String = fs::read_to_string(train_path).expect("Error loading train");
    let test_raw: String = fs::read_to_string(test_path).expect("Error loading test");

    // Load at word level.
    type E = usize;
    let mut index = TokenIndex::new();
    let mut train: Vec<usize> = train_raw.split_whitespace().map(|x| index.add(x)).collect();
    let mut test: Vec<usize> = test_raw.split_whitespace().map(|x| index.add(x)).collect();
    let eos = index.index("<eos>");

    // 
    // FIXME: Issue is no probability mass on <unk>!
    // 

    train.push(eos);
    test.push(eos);
    let old_test_len = test.len();
    // let n_test = 10000;
    // test = (&test[0..n_test]).to_vec();
    let eval_threshold = train.len() / 20;

    println!("#(train): {}", train.len());
    println!("#(test): {}/{}", test.len(), old_test_len);

    // let tokens: Vec<usize> = train_raw.split_whitespace().map(|x| index.add(x)).collect();
    // println!("#(train words): {}", tokens.len());

    let mut lms: Vec<Box<dyn LM>> = Vec::new();
    for delta in vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9].iter() {
        let maxgram = KNLM::new(format!("maxgram_kn-{}", delta), *delta, -1);
        lms.push(Box::new(maxgram));
        let ngram = KNLM::new(format!("ngram_kn-{}", delta), *delta, 4);
        lms.push(Box::new(ngram));
    }
    let mut evaluator = Evaluator::new(&lms, &test);

    let mut dawg: Dawg<E> = Dawg::new();
    let mut last = dawg.get_initial();
    for (idx, token) in tqdm!(train.iter()).enumerate() {
        last = dawg.extend(*token, last);
        if idx % eval_threshold == 0 {
            // FIXME: Use right lengths here? Shouldn't matter too much.
            let good_turing = good_turing_estimate(&dawg, train.len());        
            evaluator.evaluate(&dawg, idx, good_turing);
        }
    }
    println!("DAWG built!");
    evaluator.to_json(out_path)?;
    println!("Successfully saved to {}!", out_path);
    
    // Graph is released here, can borrow it. Very messy pattern currently lol.
    println!("Node count: {}", dawg.node_count());
    println!("Edge count: {}", dawg.edge_count());

    Ok(())
}
