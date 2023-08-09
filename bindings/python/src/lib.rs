use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

pub mod dawg;
pub mod lms;
pub mod stat_utils;

use dawg::Dawg;
use lms::kn_lm::KNLM;
// use lms::induction_lm::InductionLM;
use stat_utils::good_turing_estimate;

/// A Python module implemented in Rust.
#[pymodule]
fn rusty_dawg(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Dawg>()?;
    m.add_class::<KNLM>()?;
    m.add_wrapped(wrap_pyfunction!(good_turing_estimate))?;
    // m.add_class::<InductionLM>()?;
    Ok(())
}

