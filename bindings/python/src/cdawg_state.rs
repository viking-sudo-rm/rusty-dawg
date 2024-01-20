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
    #[new]
    pub fn new(
        state: usize,
        token: u16,
        start: usize,
        idx: usize,
        end: usize,
        target: usize,
        length: u64,
    ) -> Self {
        Self {cs: cdawg_state::CdawgState {
            state: NodeIndex::new(state),
            token,
            start,
            idx,
            end,
            target: NodeIndex::new(target),
            length,
        }}
    }

    pub fn get_length(&self) -> u64 {
        self.cs.length
    }
}