use serde::{Deserialize, Serialize};
use std::cmp::Ord;
use std::fmt::Debug;

use crate::dawg::Dawg;
use crate::graph::indexing::{DefaultIx, NodeIndex};
use crate::graph::traits::NodeRef;
use crate::memory_backing::MemoryBacking;
use crate::weight::Weight;

pub fn get_entropy<E, N, Mb>(dawg: &Dawg<E, N, DefaultIx, Mb>, state: NodeIndex) -> f64
where
    E: Eq + Ord + Serialize + for<'a> Deserialize<'a> + Copy + Debug,
    N: Weight + Serialize + for<'a> Deserialize<'a> + Clone,
    Mb: MemoryBacking<N, E, DefaultIx>,
{
    let denom = dawg.get_node(state).get_count();
    let mut sum_num = 0;
    let mut sum_prob = 0.;
    for next_state in dawg.get_graph().neighbors(state) {
        let num = dawg.get_node(next_state).get_count();
        if num > 0 {
            let prob = (num as f64) / (denom as f64);
            sum_prob -= prob * prob.log2();
            sum_num += num;
        }
    }
    if denom - sum_num > 0 {
        // Missing probability mass corresponding to <eos>
        let missing = ((denom - sum_num) as f64) / (denom as f64);
        sum_prob -= missing * missing.log2();
    }
    sum_prob
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use super::*;
    use crate::weight::DefaultWeight;

    #[test]
    fn test_get_entropy() {
        let mut dawg: Dawg<char, DefaultWeight> = Dawg::new();
        dawg.build(&['a', 'b']);
        // Approximately log_2(3)
        assert_eq!(get_entropy(&dawg, NodeIndex::new(0)), 1.584962500721156);
        assert_eq!(get_entropy(&dawg, NodeIndex::new(1)), 0.);
        assert_eq!(get_entropy(&dawg, NodeIndex::new(2)), 0.);
    }
}
