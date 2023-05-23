use petgraph::graph::NodeIndex;
use dawg::Dawg;

// use petgraph::dot::Dot;

pub fn get_entropy(state: NodeIndex, dawg: &Dawg) -> f32 {
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

#[cfg(test)]
mod tests {

    use Dawg;
    use NodeIndex;
    use stat_utils::get_entropy;

    #[test]
    fn test_get_entropy() {
        let mut dawg = Dawg::new();
        dawg.build("ab");
        // Approximately log_2(3)
        assert_eq!(get_entropy(NodeIndex::new(0), &dawg), 1.5849626);
        assert_eq!(get_entropy(NodeIndex::new(1), &dawg), 0.);
        assert_eq!(get_entropy(NodeIndex::new(2), &dawg), 0.);
    }

}