use std::marker::Copy;
use std::fs;
use std::collections::HashMap;
use std::fmt::Debug;

// For serializing JSON.
use serde::{Serialize};
use std::io::Write;

use stat_utils::*;
use dawg::Dawg;

use lms::LM;

// TODO: 
#[derive(Serialize)]
pub struct Evaluator<'a, E>
where E: Eq + serde::Serialize + Copy + Debug {
    #[serde(skip)]
    lms: &'a Vec<Box<dyn LM>>,
    #[serde(skip)]
    test: &'a Vec<E>,
    indices: Vec<usize>,
    metrics: HashMap<&'a str, Vec<f32>>,
}

impl<E> Evaluator<'_, E>
where E: Eq + serde::Serialize + Copy + Debug {

    pub fn get(&self, key: &str) -> &Vec<f32> {
        &self.metrics[key]
    }

    pub fn get_mut(&mut self, key: &str) -> &mut Vec<f32> {
        self.metrics.get_mut(key).expect("Unknown metric")
    }

    pub fn to_json(&self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json_data = serde_json::to_string(self)?;
        let mut file = fs::File::create(file_path)?;
        file.write_all(json_data.as_bytes())?;
        Ok(())
    }

}

// TODO: Generic case
impl Evaluator<'_, usize> {

    pub fn new<'a>(lms: &'a Vec<Box<dyn LM>>, test: &'a Vec<usize>) -> Evaluator<'a, usize> {
        let indices = Vec::new();
        let mut metrics = HashMap::new();

        metrics.insert("states_per_token", Vec::new());
        metrics.insert("edges_per_token", Vec::new());
        metrics.insert("suffix_lengths", Vec::new());
        metrics.insert("suffix_counts", Vec::new());
        metrics.insert("suffix_entropies", Vec::new());
        for lm in lms.iter() {
            metrics.insert(lm.get_name(), Vec::new());
        }

        Evaluator {lms, test, indices, metrics}
    }

    pub fn evaluate(&mut self, dawg: &Dawg<usize>, idx: usize, good_turing: f32) {
        // println!("=== eval@{} ===", idx);
        // println!("counts: {:?}", counts);
        // println!("{:?}", Dot::new(dawg.get_graph()));

        let mut num_tokens = 0;
        let mut cum_length = 0;
        let mut cum_count = 0;
        let mut cum_entropy = 0.;

        let mut cum_ppls: HashMap<&str, f32> = HashMap::new();
        for lm in self.lms.iter() {
            cum_ppls.insert(lm.get_name(), 0.);
        }
    
        let mut opt_state;
        let mut state = dawg.get_initial();
        let mut length = 0;
        for token_ptr in self.test.iter() {
            let token = *token_ptr;

            // Predict the perplexity of the next token before updating the state.
            for lm in self.lms.iter() {
                let logprob = -(*lm).get_probability(&dawg, state, token, good_turing).log2();
                *cum_ppls.get_mut(lm.get_name()).unwrap() += logprob;
            }

            (opt_state, length) = dawg.transition_and_count(state, token, length);
            state = opt_state.unwrap();
            cum_length += length;
            if state.index() != 0 {
                cum_count += dawg.get_weight(state).get_count();
                // cum_count += counts[state.index()];
            }
            cum_entropy += get_entropy::<usize>(dawg, state);
            num_tokens += 1;
        }
    
        self.indices.push(idx);
        self.get_mut("states_per_token").push((dawg.node_count() as f32) / (idx as f32));
        self.get_mut("edges_per_token").push((dawg.edge_count() as f32) / (idx as f32));
        self.get_mut("suffix_lengths").push((cum_length as f32) / (num_tokens as f32));
        self.get_mut("suffix_counts").push((cum_count as f32) / (num_tokens as f32));
        self.get_mut("suffix_entropies").push((cum_entropy as f32) / (num_tokens as f32));
        for (key, ppl) in cum_ppls {
            self.get_mut(key).push((ppl as f32) / (num_tokens as f32));
        }
    }

}

#[cfg(test)]
mod tests {
    use vec_graph::dot::Dot;
    use Dawg;
    use Evaluator;
    use TokenIndex;

    use LM;
    use lms::kn_lm::KNLM;

    #[test]
    fn test_timeseries_short() {
        // Max factor of train that is suffix of test, throughout train steps:
        //   Step #0: [a, , ,] => 1 / 3 
        //   Step #1: [a, ab, ] => 3 / 3
        //   Step #2: [a, ab, ] => 3 / 3
        let train_tokens = vec!["a", "b", "b"];
        let test_tokens = vec!["a", "b", "c"];

        let mut index: TokenIndex<usize> = TokenIndex::new();
        let train: Vec<_> = train_tokens.iter().map(|x| index.add(x)).collect();
        let test: Vec<_> = test_tokens.iter().map(|x| index.index(x)).collect();

        let lms: Vec<Box<dyn LM>> = Vec::new();
        let mut evaluator: Evaluator<usize> = Evaluator::new(&lms, &test);

        let mut dawg: Dawg<usize> = Dawg::new();
        let mut last = dawg.get_initial();
        for (idx, token) in train.iter().enumerate() {
            last = dawg.extend(*token, last);
            evaluator.evaluate(&dawg, idx, 0.);
        }
        assert_eq!(*evaluator.get("suffix_lengths"), vec![1./3., 1., 1.]);
        assert_eq!(*evaluator.get("suffix_counts"), vec![1./3., 2./3., 2./3.]);
    }

    #[test]
    fn test_timeseries_repeated() {
        // Max factor of train that is suffix of test, throughout train steps:
        //   Step #0: [a, a, a] => 3 / 3 
        //   Step #1: [a, aa, aa] => 5 / 3
        let train_tokens = vec!["a", "a"];
        let test_tokens = vec!["a", "a", "a"];

        let mut index: TokenIndex<usize> = TokenIndex::new();
        let train: Vec<_> = train_tokens.iter().map(|x| index.add(x)).collect();
        let test: Vec<_> = test_tokens.iter().map(|x| index.index(x)).collect();

        let mut lms: Vec<Box<dyn LM>> = Vec::new();
        let unigram = KNLM::new("unigram".to_string(), 0., 0);
        lms.push(Box::new(unigram));
        let mut evaluator: Evaluator<usize> = Evaluator::new(&lms, &test);

        let mut dawg: Dawg<usize> = Dawg::new();
        let mut last = dawg.get_initial();
        for (idx, token) in train.iter().enumerate() {
            println!("{:?}", Dot::new(dawg.get_graph()));
            last = dawg.extend(*token, last);
            evaluator.evaluate(&dawg, idx, 0.);
        }
        assert_eq!(*evaluator.get("suffix_lengths"), vec![1., 5./3.]);
        assert_eq!(*evaluator.get("suffix_counts"), vec![1., 4./3.]);
        // Is this right? This is cross-entropy/token.
        assert_eq!(*evaluator.get("unigram"), vec![1., 0.5849625]);
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