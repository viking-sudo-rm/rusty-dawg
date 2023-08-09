use pyo3::prelude::*;

use crate::dawg::Dawg;
use rusty_dawg::stat_utils;

#[pyfunction]
pub fn good_turing_estimate(dawg: &Dawg, n_tokens: usize) -> f64 {
    stat_utils::good_turing_estimate(dawg.get_dawg(), n_tokens)
}