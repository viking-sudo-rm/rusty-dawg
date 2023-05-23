use petgraph::graph::NodeIndex;
use dawg::Dawg;

pub fn get_entropy(state: NodeIndex, dawg: &Dawg, counts: &Vec<usize>) -> f32 {
    let denom = counts[state.index()];
    let mut sum_num = 0;
    let mut sum_prob = 0.;
    for next_state in dawg.get_graph().neighbors(state) {
        let num = counts[next_state.index()];
        if num > 0 {
            let prob = (num as f32) / (denom as f32);
            sum_prob -= prob * prob.log2();
            sum_num += num;
        }
    }
    if denom - sum_num > 0 {
        // Missing probability mass corresponding to <eos>
        let missing = ((denom - sum_num) as f32) / (denom as f32);
        sum_prob -= missing * missing.log2();
    }
    sum_prob
}

// // Get perplexity of a next token treating the DAWG as a backoff LM.
// pub fn get_perplexity(state: NodeIndex, next_token: char, dawg: &Dawg, counts: &Vec<usize>) -> f32 {
//     // Figure out what to do here ://
//     let denom = counts[state.index()];
//     let mut found = false;
//     loop {
//         // Look for a transition to next_token.
//     }
//     for edge in dawg.get_graph().edges(state) {
//         if edge.weight() == char {
//         }
        
//         if num > 0 {
//             let prob = (num as f32) / (denom as f32);
//             sum_prob -= prob * prob.log2();
//             sum_num += num;
//         }
//     }
//     if denom - sum_num > 0 {
//         // Missing probability mass corresponding to <eos>
//         let missing = ((denom - sum_num) as f32) / (denom as f32);
//         sum_prob -= missing * missing.log2();
//     }
//     sum_prob
// }

#[cfg(test)]
mod tests {

    use Dawg;
    use InverseFailuresMap;
    use NodeIndex;
    use stat_utils::get_entropy;

    #[test]
    fn test_get_entropy() {
        let mut dawg = Dawg::new();
        let mut map = InverseFailuresMap::new(3);
        let mut counts = vec![0; 3];
        dawg.build("ab");
        map.build(&dawg);
        map.compute_counts(&dawg, &mut counts);
        println!("counts: {:?}", counts);
        // Approximately log_2(3)
        assert_eq!(get_entropy(NodeIndex::new(0), &dawg, &counts), 1.5849626);
        assert_eq!(get_entropy(NodeIndex::new(1), &dawg, &counts), 0.);
        assert_eq!(get_entropy(NodeIndex::new(2), &dawg, &counts), 0.);
    }

}