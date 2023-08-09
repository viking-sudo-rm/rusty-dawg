// use pyo3::prelude::*;

// use crate::dawg::Dawg;
// use crate::lms::kn_lm::KNLM;
// use rusty_dawg::lms::LM;
// use rusty_dawg::lms::induction_lm;
// use rusty_dawg::graph::indexing::NodeIndex;

// #[pyclass]
// pub struct InductionLM {
//     lm: induction_lm::InductionLM,
// }

// #[pymethods]
// impl InductionLM {

//     #[new]
//     pub fn new(name: String, base_lm: &KNLM, delta: f32) -> Self {
//         let box_lm = Box::new(base_lm.clone_lm());
//         Self {lm: induction_lm::InductionLM::new(name, box_lm, delta)}
//     }

//     pub fn reset(&mut self, dawg: &Dawg) {
//         self.lm.reset(dawg.get_dawg());
//     }

//     pub fn get_probability(&self, dawg: &Dawg, label: usize, good_turing: f32) -> f32 {
//         self.lm.get_probability(dawg.get_dawg(), label, good_turing)
//     }

//     pub fn update(&mut self, dawg: &Dawg, label: usize) {
//         self.lm.update(dawg.get_dawg(), label);
//     }

// }