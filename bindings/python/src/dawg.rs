use pyo3::prelude::*;
use pyo3::types::PyType;
use std::fs;
use bincode::deserialize_from;

use rusty_dawg::dawg;
use rusty_dawg::graph::indexing::NodeIndex;
use rusty_dawg::weight::Weight;

#[pyclass]
pub struct Dawg {
    dawg: dawg::Dawg<usize>,
}

// Wrap the normal Dawg class with a Python interface.
#[pymethods]
impl Dawg {

    #[new]
    pub fn new() -> Self {
        Self {dawg: dawg::Dawg::new()}
    }

    #[classmethod]
    pub fn load(cls: &PyType, path: String) -> PyResult<Self> {
        let mut file = fs::OpenOptions::new()
            .read(true)
            .open(&path)?;
        Ok(Self {dawg: deserialize_from(&file).expect("Failed to deserialize")})
    }

    pub fn build(&mut self, text: Vec<usize>) {
        self.dawg.build(&text);
    }

    pub fn get_initial(&self) -> usize {
        self.dawg.get_initial().index()
    }

    pub fn transition(&self, state: usize, token: usize, use_failures: bool) -> Option<usize> {
        let state_index = NodeIndex::new(state);
        match self.dawg.transition(state_index, token, use_failures) {
            Some(q) => Some(q.index()),
            None => None,
        }
    }

    pub fn transition_and_count(&self, state: usize, token: usize, length: u64) -> (Option<usize>, u64) {
        let state_index = NodeIndex::new(state);
        let (new_state, new_length) = self.dawg.transition_and_count(state_index, token, length);
        match new_state {
            Some(q) => (Some(q.index()), new_length),
            None => (None, new_length),
        }
    }

    pub fn get_count(&self, state: usize) -> u64 {
        let state_index = NodeIndex::new(state);
        self.dawg.get_weight(state_index).get_count()
    }

    // Returns (State, TokenId)
    pub fn get_edges(&self, state: usize) -> Vec<(usize, usize)> {
        let state_index = NodeIndex::new(state);
        let graph = self.dawg.get_graph();
        graph.edges(state_index).map(|edge| (edge.target().index(), *edge.weight())).collect()
    }

    pub fn recompute_lengths(&mut self) {
        self.dawg.recompute_lengths();
    }

    pub fn node_count(&self) -> usize {
        self.dawg.node_count()
    }

    pub fn edge_count(&self) -> usize {
        self.dawg.edge_count()
    }

}

impl Dawg {
    pub fn get_dawg(&self) -> &dawg::Dawg<usize> {
        &self.dawg
    }
}