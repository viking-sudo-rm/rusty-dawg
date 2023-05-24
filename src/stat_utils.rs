use petgraph::graph::NodeIndex;
use dawg::Dawg;

// use petgraph::dot::Dot;

pub fn get_entropy(dawg: &Dawg, state: NodeIndex) -> f32 {
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

pub fn get_probability_exact(dawg: &Dawg, state: NodeIndex, label: char) -> f32 {
    let denom = dawg.get_weight(state).get_count();
    let num = match dawg.transition(state, label, false) {
        Some(next_state) => dawg.get_weight(next_state).get_count(),
        None => 0,
    };
    (num as f32) / (denom as f32)
}

// TODO: Wrap into some kind of objects with hyperparameters (smoothing, backoff, etc.)
pub fn get_probability_simple_smoothing(dawg: &Dawg, state: NodeIndex, label: char) -> f32 {
    let smooth_denom = dawg.get_weight(state).get_count() + 256;
    let smooth_num = match dawg.transition(state, label, false) {
        Some(next_state) => dawg.get_weight(next_state).get_count() + 1,
        None => 1,
    };
    (smooth_num as f32) / (smooth_denom as f32)
}

#[cfg(test)]
mod tests {

    use Dot;
    use Dawg;
    use NodeIndex;
    use stat_utils::*;

    #[test]
    fn test_get_entropy() {
        let mut dawg = Dawg::new();
        dawg.build("ab");
        // Approximately log_2(3)
        assert_eq!(get_entropy(&dawg, NodeIndex::new(0)), 1.5849626);
        assert_eq!(get_entropy(&dawg, NodeIndex::new(1)), 0.);
        assert_eq!(get_entropy(&dawg, NodeIndex::new(2)), 0.);
    }

    #[test]
    fn test_get_probability_exact() {
        let mut dawg = Dawg::new();
        dawg.build("ab");
        assert_eq!(get_probability_exact(&dawg, NodeIndex::new(0), 'b'), 1./3.);
        assert_eq!(get_probability_exact(&dawg, NodeIndex::new(1), 'b'), 1.);
        assert_eq!(get_probability_exact(&dawg, NodeIndex::new(2), 'b'), 0.);
    }

}