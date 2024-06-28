use pyo3::prelude::*;

use rusty_dawg::cdawg::cdawg_state;
use rusty_dawg::graph::indexing::{DefaultIx, NodeIndex};

#[pyclass]
#[derive(Clone)]
pub struct CdawgState {
    pub cs: cdawg_state::CdawgState<DefaultIx>,
}

#[pymethods]
impl CdawgState {
    /// Used in hacky way to traverse states.
    #[new]
    pub fn new(q: usize, length: u64) -> Self {
        let state = NodeIndex::new(q);
        let cs = cdawg_state::CdawgState {state, edge_start: 0, start: 0, end: 0, target: None, length};
        Self {cs}
    }

    pub fn get_length(&self) -> u64 {
        self.cs.length
    }

    pub fn get_state_and_gamma(&self) -> (Option<usize>, (usize, usize)) {
        let (state, gamma) = self.cs.get_state_and_gamma();
        match state {
            Some(q) => (Some(q.index()), gamma),
            None => (None, gamma),
        }
    }
}
