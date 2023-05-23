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
mod inverse_failures;
mod stat_utils;

// use std::cmp::max;
use std::io::{self, Read};
use std::mem::size_of;
// use std::vec;

use substring::Substring;

use petgraph::graph::NodeIndex;
use petgraph::dot::Dot;

use dawg::Dawg;
use weight::BasicWeight;
use inverse_failures::InverseFailuresMap;

// For serializing JSON.
use serde::{Serialize};
use std::fs::File;
use std::io::Write;

use kdam::tqdm;

use stat_utils::get_entropy;

#[derive(Serialize)]
struct Evaluator<'a> {
    test: &'a str,
    indices: Vec<usize>,
    suffix_lengths: Vec<f32>,
    suffix_counts: Vec<f32>,
    suffix_entropies: Vec<f32>,
    states_per_token: Vec<f32>,
    edges_per_token: Vec<f32>,
}

impl Evaluator<'_> {

    pub fn new<'a>(test: &'a str) -> Evaluator<'a> {
        let mut indices = Vec::new();
        let mut suffix_lengths = Vec::new();
        let mut suffix_counts = Vec::new();
        let mut suffix_entropies = Vec::new();
        let mut states_per_token = Vec::new();
        let mut edges_per_token = Vec::new();
        Evaluator {
            test: test,
            indices: indices,
            suffix_lengths: suffix_lengths,
            suffix_counts: suffix_counts,
            suffix_entropies: suffix_entropies,
            states_per_token: states_per_token,
            edges_per_token: edges_per_token,
        }
    }

    pub fn evaluate(&mut self, dawg: &Dawg, counts: &Vec<usize>, idx: usize) {
        // println!("=== eval@{} ===", idx);
        // println!("counts: {:?}", counts);
        // println!("{:?}", Dot::new(dawg.get_graph()));

        let mut cum_length = 0;
        let mut cum_count = 0;
        let mut cum_entropy = 0.;
        let mut num_tokens = 0;
    
        let mut opt_state = None;
        let mut state = dawg.get_initial();
        let mut length = 0;
        for token in self.test.chars() {
            (opt_state, length) = dawg.transition_and_count(state, token, length);
            state = opt_state.unwrap();
            cum_length += length;
            if state.index() != 0 {
                cum_count += counts[state.index()];
            }
            cum_entropy += get_entropy(state, dawg, counts);
            num_tokens += 1;
        }
    
        self.indices.push(idx);
        self.suffix_lengths.push((cum_length as f32) / (num_tokens as f32));
        self.suffix_counts.push((cum_count as f32) / (num_tokens as f32));
        self.suffix_entropies.push((cum_entropy as f32) / (num_tokens as f32));
        self.states_per_token.push((dawg.node_count() as f32) / (idx as f32));
        self.edges_per_token.push((dawg.edge_count() as f32) / (idx as f32));
    }

    pub fn to_json(&self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json_data = serde_json::to_string(self)?;
        let mut file = File::create(file_path)?;
        file.write_all(json_data.as_bytes())?;
        Ok(())
    }

}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Weight size: {}", size_of::<BasicWeight>());

    let stdin = io::stdin();
    let mut text = String::new();
    stdin.lock().read_to_string(&mut text).expect("Couldn't read");
    let length = text.len();
    let train = text.substring(0, length - 10000);
    let test = text.substring(length - 10000, length);

    let mut dawg = Dawg::new();
    let mut map = InverseFailuresMap::new(2 * length);
    let mut counts = vec![0; 2 * length];
    let mut evaluator = Evaluator::new(test);
    let mut last = dawg.get_initial();
    for (idx, token) in tqdm!(train.chars()).enumerate() {
        last = dawg.extend(token, last);
        if idx % 100000 == 0 {
            // FIXME: Use right lengths here? Shouldn't matter too much.
            map.clear();
            map.build(&dawg);
            map.compute_counts(&dawg, &mut counts);
            evaluator.evaluate(&dawg, &counts, idx);
        }
    }
    println!("DAWG built!");
    let path = "/Users/willm/Desktop/brown.json";
    evaluator.to_json(path)?;
    println!("Successfully saved to {}!", path);
    
    // Graph is released here, can borrow it. Very messy pattern currently lol.
    println!("Node count: {}", dawg.node_count());
    println!("Edge count: {}", dawg.edge_count());

    Ok(())
}

#[cfg(test)]
mod tests {

    use Dawg;
    use Evaluator;
    use InverseFailuresMap;

    #[test]
    fn test_timeseries_short() {
        // Max factor of train that is suffix of test, throughout train steps:
        //   Step #0: [a, , ,] => 1 / 3 
        //   Step #1: [a, ab, ] => 3 / 3
        //   Step #2: [a, ab, ] => 3 / 3
        let train = "abb";
        let test = "abc";
        let mut dawg = Dawg::new();
        let mut map = InverseFailuresMap::new(2 * train.len());
        let mut counts = vec![0; 2 * train.len()];
        let mut evaluator = Evaluator::new(test);
        let mut last = dawg.get_initial();
        for (idx, token) in train.chars().enumerate() {
            last = dawg.extend(token, last);
            map.clear();
            map.build(&dawg);
            map.compute_counts(&dawg, &mut counts);
            evaluator.evaluate(&dawg, &counts, idx);
        }
        dawg.recompute_lengths(); // FIXME
        assert_eq!(evaluator.suffix_lengths, vec![1./3., 1., 1.]);
        assert_eq!(evaluator.suffix_counts, vec![1./3., 2./3., 2./3.]);
    }

    #[test]
    fn test_timeseries_repeated() {
        // Max factor of train that is suffix of test, throughout train steps:
        //   Step #0: [a, a, a] => 3 / 3 
        //   Step #1: [a, aa, aa] => 5 / 3
        let train = "aa";
        let test = "aaa";
        let mut dawg = Dawg::new();
        let mut map = InverseFailuresMap::new(2 * train.len());
        let mut counts = vec![0; 2 * train.len()];
        let mut evaluator = Evaluator::new(test);
        let mut last = dawg.get_initial();
        for (idx, token) in train.chars().enumerate() {
            last = dawg.extend(token, last);
            map.clear();
            map.build(&dawg);
            counts = vec![0; 2 * train.len()];
            map.compute_counts(&dawg, &mut counts);
            evaluator.evaluate(&dawg, &counts, idx);
        }
        dawg.recompute_lengths();  // FIXME
        assert_eq!(evaluator.suffix_lengths, vec![1., 5./3.]);
        assert_eq!(evaluator.suffix_counts, vec![1., 4./3.]);
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