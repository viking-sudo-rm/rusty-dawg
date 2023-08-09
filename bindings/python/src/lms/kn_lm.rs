use pyo3::prelude::*;

use crate::dawg::Dawg;
use rusty_dawg::lms::LM;
use rusty_dawg::lms::kn_lm;
use rusty_dawg::graph::indexing::NodeIndex;

#[pyclass]
pub struct KNLM {
    lm: kn_lm::KNLM,
}

#[pymethods]
impl KNLM {

    #[new]
    pub fn new(name: String, kn_delta: f64, kn_max_n: i64, min_freq: u64) -> Self {
        Self {lm: kn_lm::KNLM::new(name, kn_delta, kn_max_n, min_freq)}
    }

    pub fn reset(&mut self, dawg: &Dawg) {
        self.lm.reset(dawg.get_dawg());
    }

    pub fn get_probability(&self, dawg: &Dawg, label: usize, good_turing: f64) -> f64 {
        self.lm.get_probability(dawg.get_dawg(), label, good_turing)
    }

    pub fn update(&mut self, dawg: &Dawg, label: usize) {
        self.lm.update(dawg.get_dawg(), label);
    }

}

// impl KNLM {
//     pub fn clone_lm(&self) -> kn_lm::KNLM {
//         self.lm
//     }
// }