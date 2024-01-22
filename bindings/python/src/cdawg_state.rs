use pyo3::prelude::*;

use rusty_dawg::cdawg::cdawg_state;
use rusty_dawg::graph::indexing::DefaultIx;

#[pyclass]
#[derive(Clone)]
pub struct CdawgState {
    pub cs: cdawg_state::CdawgState<DefaultIx>,
}

#[pymethods]
impl CdawgState {
    pub fn get_length(&self) -> u64 {
        self.cs.length
    }
}
