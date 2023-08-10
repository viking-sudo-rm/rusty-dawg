use std::convert::TryInto;

use dawg::Dawg;
use lms::LM;
use weight::Weight;
use std::fmt::Debug;
use std::marker::Copy;
use serde::Serialize;
use serde::Deserialize;

// use petgraph::graph::NodeIndex;
use graph::indexing::NodeIndex;
use weight::weight40::DefaultWeight;

pub struct KNLM {
    pub name: String,
    // index: &'a TokenIndex<usize>,
    // dawg: &'a Dawg<u16>,
    kn_delta: f64,
    kn_max_n: i64,
    min_count: u64, // Backoff to states that occur at least this much.
    state: NodeIndex,
}

impl<E> LM<E> for KNLM
where
    E: Eq + serde::Serialize + Ord + for<'a> Deserialize<'a> + Copy + Debug,
    {
    fn get_name(&self) -> &str {
        self.name.as_str()
    }

    fn reset(&mut self, dawg: &Dawg<E, DefaultWeight>) 
    {
        self.state = dawg.get_initial();
    }

    fn get_probability(
        &self,
        dawg: &Dawg<E, DefaultWeight>,
        label: E,
        good_turing: f64,
    ) -> f64 
    {
        let mut state = self.state;
        let _initial = dawg.get_initial();
        while dawg.get_weight(state).get_count() < self.min_count {
            match dawg.get_weight(state).get_failure() {
                Some(fstate) => state = fstate,
                None => break,
            }
        }
        self.get_probability_kn(dawg, state, label, good_turing)
    }

    fn update(&mut self, dawg: &Dawg<E, DefaultWeight>, label: E)
    {
        self.state = dawg.transition(self.state, label, true).unwrap();
    }
}

impl KNLM
    {
    pub fn new(name: String, kn_delta: f64, kn_max_n: i64, min_count: u64) -> Self {
        // The state set here is correct but also unused.
        Self {
            name,
            kn_delta,
            kn_max_n,
            state: NodeIndex::new(0),
            min_count,
        }
    }

    pub fn get_probability_exact<E>(
        &self,
        dawg: &Dawg<E, DefaultWeight>,
        state: NodeIndex,
        label: E,
    ) -> f64 
    where
    E: Eq + serde::Serialize + Ord + for<'a> Deserialize<'a> + Copy + Debug,
    {
        // FIXME: Handle <eos> here!!
        let denom = dawg.get_weight(state).get_count();
        let num = match dawg.transition(state, label, false) {
            Some(next_state) => dawg.get_weight(next_state).get_count(),
            None => 0,
        };
        (num as f64) / (denom as f64)
    }

    // pub fn get_probability_simple_smoothing(&self, dawg: &Dawg<u16>, state: NodeIndex, label: usize) -> f64 {
    //     let n_types = (index.count - 1) as u64;  // Ignore <bos>
    //     let smooth_denom = dawg.get_weight(state).get_count() + n_types;
    //     let smooth_num = match dawg.transition(state, label, false) {
    //         Some(next_state) => dawg.get_weight(next_state).get_count() + 1,
    //         None => 1,
    //     };
    //     (smooth_num as f64) / (smooth_denom as f64)
    // }

    // Backoff with Kneser-Ney smoothing
    pub fn get_probability_kn<E>(
        &self,
        dawg: &Dawg<E, DefaultWeight>,
        mut state: NodeIndex,
        label: E,
        good_turing: f64,
    ) -> f64 
    where
    E: Eq + Ord + serde::Serialize + for<'a> Deserialize<'a> + Copy + Debug,
    {
        if self.kn_max_n >= 0 {
            let n: u64 = self.kn_max_n.try_into().unwrap();
            let graph = dawg.get_graph();
            // TODO: Can make this more efficient by computing once and passing.
            while n < dawg.get_length(state) + 1 {
                match graph[state].get_failure() {
                    Some(next_state) => {
                        state = next_state;
                    }
                    None => break,
                }
            }
        }

        let count = match dawg.transition(state, label, false) {
            Some(next_state) => dawg.get_weight(next_state).get_count(),
            None => 0,
        };
        let back_count = dawg.get_graph().n_edges(state);
        let sum_count = dawg.get_weight(state).get_count();
        match dawg.get_weight(state).get_failure() {
            Some(fstate) => {
                let delta = self.kn_delta;
                let back_prob = self.get_probability_kn(dawg, fstate, label, good_turing);
                ((1. - delta) * (count as f64) + delta * (back_count as f64) * back_prob)
                    / (sum_count as f64)
            }
            None => {
                // Put some probability here on <unk> using Good-Turing estimate.
                (1. - good_turing) * self.get_probability_exact(dawg, state, label) + good_turing
            }
        }
    }
}

// #[cfg(test)]
// #[allow(unused_imports)]
// mod tests {
//     use dawg::Dawg;
//     use tokenize::{TokenIndex, Tokenize};

//     use graph::indexing::NodeIndex;
//     use graph::vec_graph::dot::Dot;

//     use lms::kn_lm::KNLM;
//     use lms::LM;

//     #[test]
//     fn test_get_probability_exact() {
//         let tokens = vec!["a", "b"];
//         let mut index: TokenIndex<u16> = TokenIndex<u16>::new();
//         let indices = tokens.iter().map(|x| index.add(x)).collect();

//         let mut dawg = Dawg::new();
//         dawg.build(&indices);

//         let lm = KNLM::new("test".to_string(), 0.0, -1, 0);
//         let b = index.index("b");
//         assert_eq!(
//             lm.get_probability_exact(&dawg, NodeIndex::new(0), b),
//             1. / 3.
//         );
//         assert_eq!(lm.get_probability_exact(&dawg, NodeIndex::new(1), b), 1.);
//         assert_eq!(lm.get_probability_exact(&dawg, NodeIndex::new(2), b), 0.);
//     }

//     #[test]
//     fn test_get_probability_kn_reduces_to_exact() {
//         let tokens = vec!["a", "b"];
//         let mut index: TokenIndex<u16> = TokenIndex<u16>::new();
//         let indices = tokens.iter().map(|x| index.add(x)).collect();

//         let mut dawg = Dawg::new();
//         dawg.build(&indices);

//         let lm = KNLM::new("test".to_string(), 0.0, -1, 0);
//         let a = index.index("a");
//         let b = index.index("b");
//         assert_eq!(
//             lm.get_probability_kn(&dawg, NodeIndex::new(0), a, 0.),
//             1. / 3.
//         );
//         assert_eq!(
//             lm.get_probability_kn(&dawg, NodeIndex::new(0), b, 0.),
//             1. / 3.
//         );
//     }

//     #[test]
//     fn test_get_probability_kn_with_delta() {
//         let tokens = vec!["a", "b"];
//         let mut index: TokenIndex<u16> = TokenIndex<u16>::new();
//         let indices = tokens.iter().map(|x| index.add(x)).collect();

//         let mut dawg = Dawg::new();
//         dawg.build(&indices);

//         let lm = KNLM::new("test".to_string(), 0.1, -1, 0);
//         let a = index.index("a");
//         let b = index.index("b");
//         let c = index.index("c");

//         let pa = lm.get_probability_kn(&dawg, NodeIndex::new(0), a, 0.);
//         let pb = lm.get_probability_kn(&dawg, NodeIndex::new(0), b, 0.);
//         let pc = lm.get_probability_kn(&dawg, NodeIndex::new(0), c, 0.);
//         // In the base case, we now just return the unigram model.
//         assert_eq!(pa + pb, 2. / 3.);
//         assert_eq!(pc, 0.);

//         // println!("{:?}", Dot::new(dawg.get_graph()));
//         let pa_a = lm.get_probability_kn(&dawg, NodeIndex::new(1), a, 0.);
//         let pb_a = lm.get_probability_kn(&dawg, NodeIndex::new(1), b, 0.);
//         let pc_a = lm.get_probability_kn(&dawg, NodeIndex::new(1), c, 0.);
//         assert!(pa_a + pb_a + 254. * pc_a <= 1.);
//         // There should be some probability on <eos>
//     }

//     #[test]
//     fn test_get_probability_kn_ngram() {
//         let tokens = vec!["a", "b"];
//         let mut index: TokenIndex<u16> = TokenIndex<u16>::new();
//         let indices = tokens.iter().map(|x| index.add(x)).collect();

//         let mut dawg = Dawg::new();
//         dawg.build(&indices);

//         let lm = KNLM::new("test".to_string(), 0.0, 1, 0);
//         let b = index.index("b");

//         let pb_a = lm.get_probability_kn(&dawg, NodeIndex::new(1), b, 0.);
//         assert_eq!(pb_a, 1. / 3.);
//     }

//     #[test]
//     fn test_get_probability_abab() {
//         let tokens = vec!["a", "b", "a", "b"];
//         let mut index: TokenIndex<u16> = TokenIndex<u16>::new();
//         let indices: Vec<_> = tokens.iter().map(|x| index.add(x)).collect();

//         let mut dawg = Dawg::new();
//         dawg.build(&indices);
//         println!("{:?}", Dot::new(dawg.get_graph()));

//         let mut lm = KNLM::new("unigram".to_string(), 0.0, 0, 0);
//         let a = index.index("a");
//         let b = index.index("b");

//         assert_eq!(
//             lm.get_probability_kn(&dawg, NodeIndex::new(0), a, 0.),
//             2. / 5.
//         );
//         assert_eq!(
//             lm.get_probability_kn(&dawg, NodeIndex::new(0), b, 0.),
//             2. / 5.
//         );

//         lm.update(&dawg, a);
//         assert_eq!(lm.get_probability(&dawg, b, 0.), 2. / 5.);
//         lm.update(&dawg, b);
//         assert_eq!(lm.get_probability(&dawg, b, 0.), 2. / 5.);
//         lm.update(&dawg, a);
//         assert_eq!(lm.get_probability(&dawg, b, 0.), 2. / 5.);
//     }

//     // TODO: Test integration between Good-Turing and get_probability_kn
// }