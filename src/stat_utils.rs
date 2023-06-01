use std::cmp::Ord;
use std::fmt::Debug;
use std::convert::TryInto;

use token_index::TokenIndex;
use dawg::Dawg;

// use petgraph::graph::NodeIndex;
use vec_graph::indexing::NodeIndex;

// use petgraph::dot::Dot;

pub fn get_entropy<E>(dawg: &Dawg<E>, state: NodeIndex) -> f32
where E: Eq + Ord + serde::Serialize + Copy + Debug {
    // let denom = counts[state.index()];
    // println!("{:?}", Dot::new(dawg.get_graph()));

    let denom = dawg.get_weight(state).get_count();
    let mut sum_num = 0;
    let mut sum_prob = 0.;
    for next_state in dawg.get_graph().neighbors(state) {
        // let num = counts[next_state.index()];
        let num = dawg.get_weight(next_state).get_count();
        if num > 0 {
            let prob = (num as f32) / (denom as f32);
            sum_prob -= prob * prob.log2();
            sum_num += num;
        }
    }
    // println!("state: {}", state.index());
    // println!("denom: {}", denom);
    // println!("sum_num: {}", sum_num);
    if denom - sum_num > 0 {
        // Missing probability mass corresponding to <eos>
        let missing = ((denom - sum_num) as f32) / (denom as f32);
        sum_prob -= missing * missing.log2();
    }
    sum_prob
}

pub fn good_turing_estimate(dawg: &Dawg<usize>, n_tokens: usize) -> f32 {
    let mut n_once = 0;
    let graph = dawg.get_graph();
    for unigram in graph.neighbors(dawg.get_initial()) {
        if graph[unigram].get_count() == 1 {
            n_once += 1;
        }
    }
    (n_once as f32) / (n_tokens as f32)
}

pub struct LM<'a> {
    pub name: String,
    index: &'a TokenIndex<usize>,
    // dawg: &'a Dawg<usize>,
    kn_delta: f32,
    kn_max_n: i64,
}

// TODO: Make LM into a trait and have different kinds of LMs.
impl<'a> LM<'a> {

    pub fn new(index: &'a TokenIndex<usize>, kn_delta: f32, kn_max_n: i64) -> Self {
        Self {name: "unnamed LM".to_string(), index, kn_delta, kn_max_n}
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn get_probability_exact(&self, dawg: &Dawg<usize>, state: NodeIndex, label: usize) -> f32 {
        let denom = dawg.get_weight(state).get_count();
        let num = match dawg.transition(state, label, false) {
            Some(next_state) => dawg.get_weight(next_state).get_count(),
            None => 0,
        };
        (num as f32) / (denom as f32)
    }

    pub fn get_probability_simple_smoothing(&self, dawg: &Dawg<usize>, state: NodeIndex, label: usize) -> f32 {
        let n_types = (self.index.count - 1) as u64;  // Ignore <bos>
        let smooth_denom = dawg.get_weight(state).get_count() + n_types;
        let smooth_num = match dawg.transition(state, label, false) {
            Some(next_state) => dawg.get_weight(next_state).get_count() + 1,
            None => 1,
        };
        (smooth_num as f32) / (smooth_denom as f32)
    }

    // Backoff with Kneser-Ney smoothing
    pub fn get_probability_kn(&self, dawg: &Dawg<usize>, mut state: NodeIndex, label: usize, good_turing: f32) -> f32 {
        if self.kn_max_n >= 0 {
            let n: u64 = self.kn_max_n.try_into().unwrap();
            let graph = dawg.get_graph();
            // TODO: Can make this more efficient by computing once and passing.
            while n < dawg.get_length(state) + 1 {
                match graph[state].get_failure() {
                    Some(next_state) => {
                        state = next_state;
                    },
                    None => {break},
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
                return ((1. - delta) * (count as f32) + delta * (back_count as f32) * back_prob) / (sum_count as f32);
            }
            None => {
                // Put some probability here on <unk> using Good-Turing estimate.
                return (1. - good_turing) * self.get_probability_exact(dawg, state, label) + good_turing;
            },
        }
    }

}

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use Dawg;
    use TokenIndex;
    use stat_utils::*;

    use vec_graph::indexing::NodeIndex;
    use vec_graph::dot::Dot;

    #[test]
    fn test_get_entropy() {
        let mut dawg = Dawg::new();
        dawg.build(&"ab".chars().collect());
        // Approximately log_2(3)
        assert_eq!(get_entropy(&dawg, NodeIndex::new(0)), 1.5849626);
        assert_eq!(get_entropy(&dawg, NodeIndex::new(1)), 0.);
        assert_eq!(get_entropy(&dawg, NodeIndex::new(2)), 0.);
    }

    #[test]
    fn test_get_probability_exact() {
        let tokens = vec!["a", "b"];
        let mut index: TokenIndex<usize> = TokenIndex::new();
        let indices = tokens.iter().map(|x| index.add(x)).collect();

        let mut dawg = Dawg::new();
        dawg.build(&indices);

        let lm = LM::new(&index, 0.0, -1);
        let b = index.index("b");
        assert_eq!(lm.get_probability_exact(&dawg, NodeIndex::new(0), b), 1./3.);
        assert_eq!(lm.get_probability_exact(&dawg, NodeIndex::new(1), b), 1.);
        assert_eq!(lm.get_probability_exact(&dawg, NodeIndex::new(2), b), 0.);
    }

    #[test]
    fn test_get_probability_kn_reduces_to_exact() {
        let tokens = vec!["a", "b"];
        let mut index: TokenIndex<usize> = TokenIndex::new();
        let indices = tokens.iter().map(|x| index.add(x)).collect();

        let mut dawg = Dawg::new();
        dawg.build(&indices);

        let lm = LM::new(&index, 0.0, -1);
        let a = index.index("a");
        let b = index.index("b");
        assert_eq!(lm.get_probability_kn(&dawg, NodeIndex::new(0), a, 0.), 1./3.);
        assert_eq!(lm.get_probability_kn(&dawg, NodeIndex::new(0), b, 0.), 1./3.);
    }

    #[test]
    fn test_get_probability_kn_with_delta() {
        let tokens = vec!["a", "b"];
        let mut index: TokenIndex<usize> = TokenIndex::new();
        let indices = tokens.iter().map(|x| index.add(x)).collect();

        let mut dawg = Dawg::new();
        dawg.build(&indices);

        let lm = LM::new(&index, 0.1, -1);
        let a = index.index("a");
        let b = index.index("b");
        let c = index.index("c");

        let pa = lm.get_probability_kn(&dawg, NodeIndex::new(0), a, 0.);
        let pb = lm.get_probability_kn(&dawg, NodeIndex::new(0), b, 0.);
        let pc = lm.get_probability_kn(&dawg, NodeIndex::new(0), c, 0.);
        // In the base case, we now just return the unigram model.
        assert_eq!(pa + pb, 2./3.);
        assert_eq!(pc, 0.);

        // println!("{:?}", Dot::new(dawg.get_graph()));
        let pa_a = lm.get_probability_kn(&dawg, NodeIndex::new(1), a, 0.);
        let pb_a = lm.get_probability_kn(&dawg, NodeIndex::new(1), b, 0.);
        let pc_a = lm.get_probability_kn(&dawg, NodeIndex::new(1), c, 0.);
        assert!(pa_a + pb_a + 254. * pc_a <= 1.);
        // There should be some probability on <eos>
    }

    #[test]
    fn test_get_probability_kn_ngram() {
        let tokens = vec!["a", "b"];
        let mut index: TokenIndex<usize> = TokenIndex::new();
        let indices = tokens.iter().map(|x| index.add(x)).collect();

        let mut dawg = Dawg::new();
        dawg.build(&indices);

        let lm = LM::new(&index, 0.0, 1);
        let b = index.index("b");
  
        let pb_a = lm.get_probability_kn(&dawg, NodeIndex::new(1), b, 0.);
        assert_eq!(pb_a, 0.33333334);
    }

    #[test]
    fn test_good_turing_estimate_ab() {
        let tokens = vec!["a", "b"];
        let mut index: TokenIndex<usize> = TokenIndex::new();
        let mut indices: Vec<_> = tokens.iter().map(|x| index.add(x)).collect();
        indices.push(index.eos());
        let mut dawg = Dawg::new();
        dawg.build(&indices);

        let good_turing = good_turing_estimate(&dawg, indices.len());
        assert_eq!(good_turing, 1.);
    }

    #[test]
    fn test_good_turing_estimate_abb() {
        let tokens = vec!["a", "b", "b"];
        let mut index: TokenIndex<usize> = TokenIndex::new();
        let mut indices: Vec<_> = tokens.iter().map(|x| index.add(x)).collect();
        indices.push(index.eos());
        let mut dawg = Dawg::new();
        dawg.build(&indices);

        // println!("{:?}", Dot::new(dawg.get_graph()));
        let good_turing = good_turing_estimate(&dawg, indices.len());
        assert_eq!(good_turing, 2./4.);
    }

    // TODO: Test integration between Good-Turing and get_probability_kn

}