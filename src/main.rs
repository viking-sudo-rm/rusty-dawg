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

mod weight;

use std::cmp::max;
use std::io::{self, Read};
use std::mem::size_of;
use std::vec;

use substring::Substring;

use petgraph::Graph;
use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;
use petgraph::dot::Dot;

// For serializing JSON.
use serde::{Serialize};
use std::fs::File;
use std::io::Write;

use kdam::tqdm;

use weight::BasicWeight;

struct Dawg {
    dawg: Graph<BasicWeight, char>,
    initial: NodeIndex,
}

impl Dawg {

    pub fn new() -> Dawg {
        //dawg: &'a mut Graph<BasicWeight, char>
        // let weight = Weight::create::<W>(0, 0, None);
        let mut dawg = Graph::<BasicWeight, char>::new();
        let initial = dawg.add_node(BasicWeight::create(0, 0, None));
        Dawg {dawg: dawg, initial: initial}
    }

    pub fn build(&mut self, text: &str) {
        let mut last = self.initial;
        for token in tqdm!(text.chars()) {
            last = self.extend(token, last);
        }
    }

    pub fn extend(&mut self, token: char, last: NodeIndex) -> NodeIndex {
        let new = self.dawg.add_node(BasicWeight::extend(&self.dawg[last]));
        // Follow failure path from last until transition is defined.
        let mut opt_state = Some(last);
        let mut opt_next_state: Option<NodeIndex> = None;
        // println!("last: {:?}", last);
        loop {
            let state = opt_state.unwrap();
            self.dawg.add_edge(state, new, token);
            opt_state = self.dawg[state].get_failure();
            match opt_state {
                Some(state) => {
                    opt_next_state = self.transition(state, token, false);
                    // println!("next: {:?}", opt_next_state);
                    if opt_next_state.is_some() {
                        break;
                    }
                },
                None => break,
            }
        }
    
        // println!("===============");
        // println!("Token: {}", token);
        // println!("New: {:?}", new);
        // println!("State: {:?}", opt_state);
        // if opt_state.is_some() {
        //     println!("Fail: {:?}", dawg[opt_state.unwrap()].failure);
        // }
        // println!("\n\n");
    
        match opt_state {
            // There is no valid failure state for the new state.
            None => self.dawg[new].set_failure(Some(self.initial)),
    
            // Found a failure state to fail to.
            Some(mut state) => {
                let next_state = opt_next_state.unwrap();
                if self.dawg[state].get_length() + 1 == self.dawg[next_state].get_length() {
                    // Fail to an existing state.
                    self.dawg[new].set_failure(Some(next_state));
                }
                
                else {
                    // Split a state and fail to the cl1 of it.
                    let cl1 = self.dawg.add_node(BasicWeight::create(
                        0,
                        self.dawg[state].get_length() + 1,
                        self.dawg[next_state].get_failure(),
                    ));
                    // TODO: Could possibly avoid collecting here.
                    let edges: Vec<_> = self.dawg.edges(next_state).map(|edge| (edge.target(), *edge.weight())).collect();
                    for (target, weight) in edges {
                        self.dawg.add_edge(cl1, target, weight);
                    }
                    self.dawg[new].set_failure(Some(cl1));
                    self.dawg[next_state].set_failure(Some(cl1));
    
                    // Reroute edges along failure chain.
                    let mut next_state_ = next_state;
                    loop {
                        if next_state_ == next_state {
                            let edge = self.dawg.find_edge(state, next_state_).unwrap();
                            self.dawg.remove_edge(edge);
                        }
                        self.dawg.add_edge(state, cl1, token);
    
                        match self.dawg[state].get_failure() {
                            None => break,
                            Some(q) => {
                                state = q;
                            },
                        }
                        match self.transition(state, token, false) {
                            Some(value) => {
                                next_state_ = value;
                                if next_state_ != next_state {
                                    break;
                                }
                            },
                            None => {},
                        }
                    }
                }
            },
        }
        return new;
    }

    pub fn recompute_lengths(&mut self) {
        self._zero_lengths(self.initial);
        self._recompute_lengths(self.initial, 0);
    }

    fn _zero_lengths(&mut self, state: NodeIndex) {
        self.dawg[state].set_length(0);
        let next_states: Vec<_> = self.dawg.neighbors(state).collect();
        for next_state in next_states {
            self._zero_lengths(next_state);
        }    }

    fn _recompute_lengths(&mut self, state: NodeIndex, length: u64) {
        self.dawg[state].set_length(length);
        let next_states: Vec<_> = self.dawg.neighbors(state).collect();
        for next_state in next_states {
            if self.dawg[next_state].get_length() == 0 {
                self._recompute_lengths(next_state, length + 1);
            }
        }
    }

    fn transition(&self, state: NodeIndex, token: char, use_failures: bool) -> Option<NodeIndex> {
        // TODO(willm): Could implement binary search over sorted edges here.
        for edge in self.dawg.edges(state) {
            if token == *edge.weight() {
                return Some(edge.target());
            }
        }
    
        if !use_failures {
            return None;
        }
        let fail_state = self.dawg[state].get_failure();
        match fail_state {
            Some(q) => {
                // Not in the initial state.
                return self.transition(q, token, use_failures);
            },
            // Only possible in the initial state.
            None => return Some(self.initial),
        }    
    }

    //Return the length of the largest matching suffix.
    fn transition_and_count(&self, state: NodeIndex, token: char, length: u64) -> (Option<NodeIndex>, u64) {
        // TODO(willm): Could implement binary search over sorted edges here.
        for edge in self.dawg.edges(state) {
            if token == *edge.weight() {
                return (Some(edge.target()), length + 1);
            }
        }
    
        let fail_state = self.dawg[state].get_failure();
        match fail_state {
            Some(q) => {
                let new_length = self.dawg[q].get_length();
                return self.transition_and_count(q, token, new_length);
            },
            // Only possible in the initial state.
            None => return (Some(self.initial), 0),
        }
    }

    // Return the length of the largest substring of query that appears in the corpus.
    pub fn get_max_factor_length(&self, query: &str) -> u64 {
        let mut opt_state = None;
        let mut state = self.initial;
        let mut length = 0;
        let mut max_length = 0;
        for token in query.chars() {
            (opt_state, length) = self.transition_and_count(state, token, length);
            state = opt_state.unwrap();
            max_length = max(max_length, length);
        }
        return max_length;
    }

    // TODO: Get counts.

    // TODO: Can build full substring vector for query.

}

#[derive(Serialize)]
struct Evaluator<'a> {
    test: &'a str,
    indices: Vec<usize>,
    suffix_lengths: Vec<f32>,
    states_per_token: Vec<f32>,
    edges_per_token: Vec<f32>,
}

impl Evaluator<'_> {

    pub fn new<'a>(test: &'a str) -> Evaluator<'a> {
        let mut indices = Vec::new();
        let mut suffix_lengths = Vec::new();
        let mut states_per_token = Vec::new();
        let mut edges_per_token = Vec::new();
        Evaluator {
            test: test,
            indices: indices,
            suffix_lengths: suffix_lengths,
            states_per_token: states_per_token,
            edges_per_token: edges_per_token,
        }
    }

    pub fn evaluate(&mut self, dawg: &Dawg, idx: usize) {    
        let mut cum_length = 0;
        let mut num_tokens = 0;
    
        let mut opt_state = None;
        let mut state = dawg.initial;
        let mut length = 0;
        // println!("eval at {}", idx);
        // println!("{:?}", Dot::new(&dawg.dawg));
        for token in self.test.chars() {
            (opt_state, length) = dawg.transition_and_count(state, token, length);
            state = opt_state.unwrap();
            cum_length += length;
            num_tokens += 1;
        }
    
        self.indices.push(idx);
        self.suffix_lengths.push((cum_length as f32) / (num_tokens as f32));
        self.states_per_token.push((dawg.dawg.node_count() as f32) / (idx as f32));
        self.edges_per_token.push((dawg.dawg.edge_count() as f32) / (idx as f32));
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
    let train = text.substring(0, length - 1000);
    let test = text.substring(length - 1000, length);

    // let args: Vec<String> = env::args().collect();
    // let text = &args[1].as_str();

    // TODO: Wrap these in module, create dawg in constructor.
    // let mut graph = Graph::<BasicWeight, char>::new();
    // let mut dawg = Dawg::create(&mut graph);
    let mut dawg = Dawg::new();
    let mut evaluator = Evaluator::new(test);
    let mut last = dawg.initial;
    for (idx, token) in tqdm!(train.chars().enumerate()) {
        last = dawg.extend(token, last);
        if idx % 10000 == 0 {
            evaluator.evaluate(&dawg, idx);
        }
    }
    println!("DAWG built!");
    evaluator.to_json("/Users/willm/Desktop/dawg-data.json")?;
    println!("Successfully wrote file!");
    
    // Graph is released here, can borrow it. Very messy pattern currently lol.
    println!("Node count: {}", dawg.dawg.node_count());
    println!("Edge count: {}", dawg.dawg.edge_count());

    Ok(())
}

#[cfg(test)]
mod tests {

    use Graph;
    use Dawg;
    use BasicWeight;
    use Evaluator;
    use Dot;
    use NodeIndex;

    #[test]
    fn test_build_bab() {
        let mut dawg = Dawg::new();
        dawg.build("bab");
        dawg.recompute_lengths();
        // println!("{:?}", Dot::new(&dawg.dawg));
        assert_eq!(dawg.dawg[NodeIndex::new(0)].get_length(), 0);
        assert_eq!(dawg.dawg[NodeIndex::new(1)].get_length(), 1);
        assert_eq!(dawg.dawg[NodeIndex::new(2)].get_length(), 1);
        assert_eq!(dawg.dawg[NodeIndex::new(3)].get_length(), 2);

        assert_eq!(dawg.get_max_factor_length("ab"), 2);
        assert_eq!(dawg.get_max_factor_length("bb"), 1);
        assert_eq!(dawg.get_max_factor_length("ba"), 2);
        assert_eq!(dawg.get_max_factor_length("z"), 0);
    }

    #[test]
    fn test_build_abcab() {
        let mut dawg = Dawg::new();
        dawg.build("abcab");
        dawg.recompute_lengths();
        assert_eq!(dawg.get_max_factor_length("ab"), 2);
        assert_eq!(dawg.get_max_factor_length("abc"), 3);
        assert_eq!(dawg.get_max_factor_length("ca"), 2);
        assert_eq!(dawg.get_max_factor_length("z"), 0);
        assert_eq!(dawg.get_max_factor_length("zzbcazz"), 3);
    }

    #[test]
    fn test_build_brown() {
        let corpus = "Communication
        may be facilitated by means of the high visibility within the larger
        community. Intense interaction is easier where segregated living and
        occupational segregation mark off a group from the rest of the community,
        as in the case of this population. However, the factor of physical  
        isolation is not a static situation. Although the Brandywine population
        is still predominantly rural, there are indications of a consistent
        and a statistically significant trend away from the older and
        relatively isolated rural communities **h urbanization appears to be";
        let mut dawg = Dawg::new();
        dawg.build(corpus);
        dawg.recompute_lengths();
        assert_eq!(dawg.get_max_factor_length("How"), 3);
        assert_eq!(dawg.get_max_factor_length("However,"), 8);
        assert_eq!(dawg.get_max_factor_length("static~However, the farce"), 15);
        assert_eq!(dawg.get_max_factor_length("However, the zzz"), 13);
    }

    #[test]
    fn test_timeseries_short() {
        let train = "abb";
        let test = "abc";
        let mut dawg = Dawg::new();
        let mut evaluator = Evaluator::new(test);
        let mut last = dawg.initial;
        for (idx, token) in train.chars().enumerate() {
            last = dawg.extend(token, last);
            evaluator.evaluate(&dawg, idx);
        }
        dawg.recompute_lengths();
        // Max factor of train that is suffix of test:
        //   Step #0: [a, , ,] => 1 / 3 
        //   Step #1: [a, ab, ] => 3 / 3
        //   Step #2: [a, ab, ] => 3 / 3
        assert_eq!(evaluator.suffix_lengths, vec![1./3., 1., 1.]);
    }

    #[test]
    fn test_timeseries_repeated() {
        let train = "aa";
        let test = "aaa";
        let mut dawg = Dawg::new();
        let mut evaluator = Evaluator::new(test);
        let mut last = dawg.initial;
        for (idx, token) in train.chars().enumerate() {
            last = dawg.extend(token, last);
            evaluator.evaluate(&dawg, idx);
        }
        dawg.recompute_lengths();
        assert_eq!(evaluator.suffix_lengths, vec![1., 5./3.]);
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