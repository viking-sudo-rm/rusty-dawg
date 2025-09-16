use crate::dawg::Dawg;
use crate::graph::indexing::DefaultIx;
use crate::graph::traits::NodeRef;
use crate::memory_backing::MemoryBacking;
use crate::stat_utils::get_entropy;
use crate::weight::Weight;
use serde::{Deserialize, Serialize};
use std::cmp::max;
use std::cmp::Ord;
use std::collections::HashMap;
use std::fmt::Debug;
use std::fs;
use std::io::Write;
use std::marker::Copy;

#[derive(Serialize)]
pub struct Evaluator<'a, E>
where
    E: Eq + serde::Serialize + Copy + Debug,
{
    #[serde(skip)]
    test: &'a Vec<E>,
    indices: Vec<usize>,
    metrics: HashMap<String, Vec<f64>>,
    max_length: u64,
}

impl<E> Evaluator<'_, E>
where
    E: Eq + Ord + serde::Serialize + Copy + Debug,
{
    pub fn get(&self, key: &str) -> &Vec<f64> {
        &self.metrics[key]
    }

    pub fn get_mut(&mut self, key: String) -> &mut Vec<f64> {
        self.metrics.get_mut(&key).expect("Unknown metric")
    }

    pub fn to_json(&self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json_data = serde_json::to_string(self)?;
        let mut file = fs::File::create(file_path)?;
        file.write_all(json_data.as_bytes())?;
        Ok(())
    }
}

// TODO: Generic case
impl<E> Evaluator<'_, E>
where
    E: Eq + Ord + serde::Serialize + for<'a> Deserialize<'a> + Copy + Debug,
{
    pub fn new(test: &Vec<E>, max_length: u64) -> Evaluator<'_, E> {
        let indices = Vec::new();
        let mut metrics = HashMap::new();

        metrics.insert("states_per_token".to_string(), Vec::new());
        metrics.insert("edges_per_token".to_string(), Vec::new());
        metrics.insert("suffix_lengths".to_string(), Vec::new());
        metrics.insert("max_suffix_lengths".to_string(), Vec::new());
        metrics.insert("suffix_counts".to_string(), Vec::new());
        metrics.insert("suffix_entropies".to_string(), Vec::new());
        for length in 0..max_length + 1 {
            metrics.insert(format!("length{}_count", length), Vec::new());
        }
        metrics.insert("length+_count".to_string(), Vec::new());

        Evaluator {
            test,
            indices,
            metrics,
            max_length,
        }
    }

    pub fn evaluate<W, Mb>(&mut self, dawg: &Dawg<E, W, DefaultIx, Mb>, idx: usize)
    where
        W: Weight + Serialize + for<'a> Deserialize<'a> + Clone,
        Mb: MemoryBacking<W, E, DefaultIx>,
    {
        let mut num_tokens = 0;
        let mut cum_length = 0;
        let mut cum_count = 0;
        let mut cum_entropy = 0.;
        let mut max_length = 0;

        let mut opt_state;
        let mut state = dawg.get_initial();
        let mut length = 0;

        for length in 0..self.max_length + 1 {
            self.get_mut(format!("length{}_count", length)).push(0.);
        }
        self.get_mut("length+_count".to_string()).push(0.);
        let it = self.metrics.get("length+_count").unwrap().len() - 1;

        for token_ptr in self.test.iter() {
            let token = *token_ptr;
            (opt_state, length) = dawg.transition_and_count(state, token, length);
            state = opt_state.unwrap();
            cum_length += length;
            max_length = max(max_length, length);
            if length <= self.max_length {
                self.get_mut(format!("length{}_count", length))[it] += 1.;
            } else {
                self.get_mut("length+_count".to_string())[it] += 1.;
            }
            if state.index() != 0 {
                cum_count += dawg.get_node(state).get_count();
                // cum_count += counts[state.index()];
            }
            cum_entropy += get_entropy::<E, W, Mb>(dawg, state);
            num_tokens += 1;
        }

        self.indices.push(idx);
        self.get_mut("states_per_token".to_string())
            .push((dawg.node_count() as f64) / (idx as f64));
        self.get_mut("edges_per_token".to_string())
            .push((dawg.edge_count() as f64) / (idx as f64));
        self.get_mut("suffix_lengths".to_string())
            .push((cum_length as f64) / (num_tokens as f64));
        self.get_mut("max_suffix_lengths".to_string())
            .push(max_length as f64);
        self.get_mut("suffix_counts".to_string())
            .push((cum_count as f64) / (num_tokens as f64));
        self.get_mut("suffix_entropies".to_string())
            .push(cum_entropy / (num_tokens as f64));
    }
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use crate::dawg::Dawg;
    use crate::evaluator::Evaluator;
    use crate::graph::indexing::DefaultIx;
    use crate::memory_backing::RamBacking;
    use crate::tokenize::{TokenIndex, Tokenize};
    use crate::weight::weight40::DefaultWeight;

    #[test]
    fn test_timeseries_short() {
        // Max factor of train that is suffix of test, throughout train steps:
        //   Step #0: [a, , ,] => 1 / 3
        //   Step #1: [a, ab, ] => 3 / 3
        //   Step #2: [a, ab, ] => 3 / 3
        let train_tokens = ["a", "b", "b"];
        let test_tokens = ["a", "b", "c"];

        let mut index: TokenIndex<u16> = TokenIndex::new();
        let train: Vec<_> = train_tokens.iter().map(|x| index.add(x)).collect();
        let test: Vec<_> = test_tokens.iter().map(|x| index.index(x)).collect();

        let mut evaluator: Evaluator<u16> = Evaluator::new(&test, 3);
        let mut dawg: Dawg<u16, DefaultWeight> = Dawg::new();
        let mut last = dawg.get_initial();
        let mut length = 0;
        for (idx, token) in train.iter().enumerate() {
            (last, length) = dawg.extend(*token, last, length);
            evaluator.evaluate(&dawg, idx);
        }
        assert_eq!(*evaluator.get("suffix_lengths"), vec![1. / 3., 1., 1.]);
        assert_eq!(*evaluator.get("length0_count"), vec![2., 1., 1.]);
        assert_eq!(*evaluator.get("length1_count"), vec![1., 1., 1.]);
        assert_eq!(*evaluator.get("length2_count"), vec![0., 1., 1.]);
        assert_eq!(*evaluator.get("length3_count"), vec![0., 0., 0.]);
        assert_eq!(
            *evaluator.get("suffix_counts"),
            vec![1. / 3., 2. / 3., 2. / 3.]
        );
    }

    #[test]
    fn test_timeseries_repeated() {
        // Max factor of train that is suffix of test, throughout train steps:
        //   Step #0: [a, a, a] => 3 / 3
        //   Step #1: [a, aa, aa] => 5 / 3
        let train_tokens = ["a", "a"];
        let test_tokens = ["a", "a", "a"];

        let mut index: TokenIndex<u16> = TokenIndex::new();
        let train: Vec<_> = train_tokens.iter().map(|x| index.add(x)).collect();
        let test: Vec<_> = test_tokens.iter().map(|x| index.index(x)).collect();

        let mut evaluator: Evaluator<u16> = Evaluator::new(&test, 3);
        let mut dawg: Dawg<u16, DefaultWeight> = Dawg::new();
        let mut last = dawg.get_initial();
        let mut length = 0;
        for (idx, token) in train.iter().enumerate() {
            (last, length) = dawg.extend(*token, last, length);
            evaluator.evaluate(&dawg, idx);
        }
        assert_eq!(*evaluator.get("suffix_lengths"), vec![1., 5. / 3.]);
        assert_eq!(*evaluator.get("suffix_counts"), vec![1., 4. / 3.]);
    }
}
