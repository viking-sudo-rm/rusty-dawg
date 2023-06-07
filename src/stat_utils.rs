use std::cmp::Ord;
use std::fmt::Debug;
use serde::{Serialize, Deserialize};

use dawg::Dawg;
// use petgraph::graph::NodeIndex;
use vec_graph::indexing::NodeIndex;

pub fn get_entropy<E>(dawg: &Dawg<E>, state: NodeIndex) -> f32
where E: Eq + Ord + Serialize + for<'a> Deserialize<'a> + Copy + Debug {
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

}