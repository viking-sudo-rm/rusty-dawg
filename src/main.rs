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
mod custom_graph;

// use std::cmp::max;
// use std::io::{self, Read};
use std::mem::size_of;
use std::marker::Copy;
use std::fs;
use std::collections::HashMap;
// use std::vec;
// use substring::Substring;

// For serializing JSON.
use serde::{Serialize};
use std::fs::File;
use std::io::Write;

use kdam::tqdm;

use stat_utils::*;
use dawg::Dawg;
use weight::BasicWeight;
use token_index::TokenIndex;

#[derive(Serialize)]
struct Evaluator<'a, E: Eq + serde::Serialize + Copy> {
    #[serde(skip)]
    test: &'a Vec<E>,
    indices: Vec<usize>,
    metrics: HashMap<&'a str, Vec<f32>>,
}

impl<E: Eq + serde::Serialize + Copy> Evaluator<'_, E> {

    pub fn new<'a>(test: &'a Vec<E>) -> Evaluator<'a, E> {
        let indices = Vec::new();
        let mut metrics = HashMap::new();
        metrics.insert("suffix_lengths", Vec::new());
        metrics.insert("suffix_counts", Vec::new());
        metrics.insert("suffix_entropies", Vec::new());
        metrics.insert("test_ppls_kn5", Vec::new());
        metrics.insert("test_ppls_kn4", Vec::new());
        metrics.insert("test_ppls_kn3", Vec::new());
        metrics.insert("test_ppls_kn2", Vec::new());
        metrics.insert("test_ppls_kn1", Vec::new());
        metrics.insert("test_ppls_kn01", Vec::new());
        metrics.insert("test_ppls_kn5_max4", Vec::new());
        metrics.insert("test_ppls_kn4_max4", Vec::new());
        metrics.insert("test_ppls_kn3_max4", Vec::new());
        metrics.insert("test_ppls_kn2_max4", Vec::new());
        metrics.insert("test_ppls_kn1_max4", Vec::new());
        metrics.insert("test_ppls_kn01_max4", Vec::new());
        metrics.insert("states_per_token", Vec::new());
        metrics.insert("edges_per_token", Vec::new());

        Evaluator {
            test: test,
            indices: indices,
            metrics: metrics,
        }
    }

    pub fn get(&self, key: &str) -> &Vec<f32> {
        &self.metrics[key]
    }

    pub fn get_mut(&mut self, key: &str) -> &mut Vec<f32> {
        self.metrics.get_mut(key).expect("Unknown metric")
    }

    pub fn evaluate(&mut self, dawg: &Dawg<E>, idx: usize) {
        // println!("=== eval@{} ===", idx);
        // println!("counts: {:?}", counts);
        // println!("{:?}", Dot::new(dawg.get_graph()));

        let mut num_tokens = 0;
        let mut cum_length = 0;
        let mut cum_count = 0;
        let mut cum_entropy = 0.;

        let mut cum_test_ppl_kn5 = 0.;
        let mut cum_test_ppl_kn4 = 0.;
        let mut cum_test_ppl_kn3 = 0.;
        let mut cum_test_ppl_kn2 = 0.;
        let mut cum_test_ppl_kn1 = 0.;
        let mut cum_test_ppl_kn01 = 0.;

        let mut cum_test_ppl_kn5_max4 = 0.;
        let mut cum_test_ppl_kn4_max4 = 0.;
        let mut cum_test_ppl_kn3_max4 = 0.;
        let mut cum_test_ppl_kn2_max4 = 0.;
        let mut cum_test_ppl_kn1_max4 = 0.;
        let mut cum_test_ppl_kn01_max4 = 0.;
    
        let mut opt_state;
        let mut state = dawg.get_initial();
        let mut length = 0;
        for token_ptr in self.test.iter() {
            let token = *token_ptr;

            // Predict the perplexity of the next token before updating the state.
            cum_test_ppl_kn5 += -get_probability_kn::<E>(dawg, state, token, 0.5, 0).log2();
            cum_test_ppl_kn4 += -get_probability_kn::<E>(dawg, state, token, 0.4, 0).log2();
            cum_test_ppl_kn3 += -get_probability_kn::<E>(dawg, state, token, 0.3, 0).log2();
            cum_test_ppl_kn2 += -get_probability_kn::<E>(dawg, state, token, 0.2, 0).log2();
            cum_test_ppl_kn1 += -get_probability_kn::<E>(dawg, state, token, 0.1, 0).log2();
            cum_test_ppl_kn01 += -get_probability_kn::<E>(dawg, state, token, 0.01, 0).log2();

            let max_n = 4;
            cum_test_ppl_kn5_max4 += -get_probability_kn::<E>(dawg, state, token, 0.5, max_n).log2();
            cum_test_ppl_kn4_max4 += -get_probability_kn::<E>(dawg, state, token, 0.4, max_n).log2();
            cum_test_ppl_kn3_max4 += -get_probability_kn::<E>(dawg, state, token, 0.3, max_n).log2();
            cum_test_ppl_kn2_max4 += -get_probability_kn::<E>(dawg, state, token, 0.2, max_n).log2();
            cum_test_ppl_kn1_max4 += -get_probability_kn::<E>(dawg, state, token, 0.1, max_n).log2();
            cum_test_ppl_kn01_max4 += -get_probability_kn::<E>(dawg, state, token, 0.01, max_n).log2();

            (opt_state, length) = dawg.transition_and_count(state, token, length);
            state = opt_state.unwrap();
            cum_length += length;
            if state.index() != 0 {
                cum_count += dawg.get_weight(state).get_count();
                // cum_count += counts[state.index()];
            }
            cum_entropy += get_entropy::<E>(dawg, state);
            num_tokens += 1;
        }
    
        self.indices.push(idx);
        self.get_mut("suffix_lengths").push((cum_length as f32) / (num_tokens as f32));
        self.get_mut("suffix_counts").push((cum_count as f32) / (num_tokens as f32));
        self.get_mut("suffix_entropies").push((cum_entropy as f32) / (num_tokens as f32));

        self.get_mut("test_ppls_kn5").push((cum_test_ppl_kn5 as f32) / (num_tokens as f32));
        self.get_mut("test_ppls_kn4").push((cum_test_ppl_kn4 as f32) / (num_tokens as f32));
        self.get_mut("test_ppls_kn3").push((cum_test_ppl_kn3 as f32) / (num_tokens as f32));
        self.get_mut("test_ppls_kn2").push((cum_test_ppl_kn2 as f32) / (num_tokens as f32));
        self.get_mut("test_ppls_kn1").push((cum_test_ppl_kn1 as f32) / (num_tokens as f32));
        self.get_mut("test_ppls_kn01").push((cum_test_ppl_kn01 as f32) / (num_tokens as f32));

        self.get_mut("test_ppls_kn5_max4").push((cum_test_ppl_kn5_max4 as f32) / (num_tokens as f32));
        self.get_mut("test_ppls_kn4_max4").push((cum_test_ppl_kn4_max4 as f32) / (num_tokens as f32));
        self.get_mut("test_ppls_kn3_max4").push((cum_test_ppl_kn3_max4 as f32) / (num_tokens as f32));
        self.get_mut("test_ppls_kn2_max4").push((cum_test_ppl_kn2_max4 as f32) / (num_tokens as f32));
        self.get_mut("test_ppls_kn1_max4").push((cum_test_ppl_kn1_max4 as f32) / (num_tokens as f32));
        self.get_mut("test_ppls_kn01_max4").push((cum_test_ppl_kn01_max4 as f32) / (num_tokens as f32));

        self.get_mut("states_per_token").push((dawg.node_count() as f32) / (idx as f32));
        self.get_mut("edges_per_token").push((dawg.edge_count() as f32) / (idx as f32));
    }

    pub fn to_json(&self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json_data = serde_json::to_string(self)?;
        let mut file = File::create(file_path)?;
        file.write_all(json_data.as_bytes())?;
        Ok(())
    }

}

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

    // Load at character level.
    // type E = char;
    // let mut train: Vec<char> = train_raw.chars().collect();
    // let mut test: Vec<char> = test_raw.chars().collect();
    // let eos = '⁂';

    // Load at word level.
    type E = usize;
    let mut index = TokenIndex::new();
    let mut train: Vec<usize> = train_raw.split_whitespace().map(|x| index.add(x)).collect();
    let mut test: Vec<usize> = test_raw.split_whitespace().map(|x| index.add(x)).collect();
    let eos = index.index("<eos>");

    train.push(eos);
    test.push(eos);
    let n_test = 10000;
    let old_test_len = test.len();
    test = (&test[0..n_test]).to_vec();
    let eval_threshold = train.len() / 20;

    println!("#(train): {}", train.len());
    println!("#(test): {}/{}", test.len(), old_test_len);

    // let tokens: Vec<usize> = train_raw.split_whitespace().map(|x| index.add(x)).collect();
    // println!("#(train words): {}", tokens.len());

    let mut dawg: Dawg<E> = Dawg::new();
    let mut evaluator = Evaluator::new(&test);
    let mut last = dawg.get_initial();
    for (idx, token) in tqdm!(train.iter()).enumerate() {
        last = dawg.extend(*token, last);
        if idx % eval_threshold == 0 {
            // FIXME: Use right lengths here? Shouldn't matter too much.
            evaluator.evaluate(&dawg, idx);
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

#[cfg(test)]
mod tests {

    use Dawg;
    use Evaluator;

    #[test]
    fn test_timeseries_short() {
        // Max factor of train that is suffix of test, throughout train steps:
        //   Step #0: [a, , ,] => 1 / 3 
        //   Step #1: [a, ab, ] => 3 / 3
        //   Step #2: [a, ab, ] => 3 / 3
        let train: Vec<char> = "abb".chars().collect();
        let test: Vec<char> = "abc".chars().collect();
        let mut dawg: Dawg<char> = Dawg::new();
        let mut evaluator: Evaluator<char> = Evaluator::new(&test);
        let mut last = dawg.get_initial();
        for (idx, token) in train.iter().enumerate() {
            last = dawg.extend(*token, last);
            evaluator.evaluate(&dawg, idx);
        }
        assert_eq!(*evaluator.get("suffix_lengths"), vec![1./3., 1., 1.]);
        assert_eq!(*evaluator.get("suffix_counts"), vec![1./3., 2./3., 2./3.]);
    }

    #[test]
    fn test_timeseries_repeated() {
        // Max factor of train that is suffix of test, throughout train steps:
        //   Step #0: [a, a, a] => 3 / 3 
        //   Step #1: [a, aa, aa] => 5 / 3
        let train: Vec<char> = "aa".chars().collect();
        let test: Vec<char> = "aaa".chars().collect();
        let mut dawg: Dawg<char> = Dawg::new();
        let mut evaluator: Evaluator<char> = Evaluator::new(&test);
        let mut last = dawg.get_initial();
        for (idx, token) in train.iter().enumerate() {
            last = dawg.extend(*token, last);
            evaluator.evaluate(&dawg, idx);
        }
        assert_eq!(*evaluator.get("suffix_lengths"), vec![1., 5./3.]);
        assert_eq!(*evaluator.get("suffix_counts"), vec![1., 4./3.]);
    }

    // #[test]
    // fn test_timeseries_brown() {
    //     let train = "of thetheir";
    //     let test = "of their";
    //     let mut dawg = Dawg::new();
    //     let mut evaluator = Evaluator::new(test);
    //     let mut last = dawg.initial;
    //     for (idx, token) in train.chars().enumerate() {
    //         last = dawg.extend(token, last);
    //         evaluator.evaluate(&dawg, idx);
    //     }
    //     // Max factor of train that is suffix of test:
    //     //   Step #0: [a, , ,] => 1 / 3 
    //     //   Step #1: [a, ab, ,] => 2 / 3
    //     //   Step #2: [a, ab, ,] => 2 / 3
    //     assert_eq!(evaluator.suffix_lengths, vec![1./8., 3./16., 6./24., 10./32., 15./40., 21./56., 28./63.]);
    // }

}