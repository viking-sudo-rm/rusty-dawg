use pyo3::prelude::*;
use pyo3::types::PyType;

use rusty_dawg::dawg;
use rusty_dawg::graph::indexing::DefaultIx;
use rusty_dawg::graph::indexing::NodeIndex;
use rusty_dawg::graph::{EdgeRef, NodeRef};
use rusty_dawg::memory_backing::{CacheConfig, DiskBacking};
use rusty_dawg::weight::DefaultWeight;

type Mb = DiskBacking<DefaultWeight, u16, DefaultIx>;

#[pyclass]
// #[pyclass(unsendable)]
pub struct DiskDawg {
    dawg: dawg::Dawg<u16, DefaultWeight, DefaultIx, Mb>,
}

// Wrap the normal Dawg class with a Python interface.
#[pymethods]
impl DiskDawg {
    #[classmethod]
    // #[pyo3(signature = (path, **kwargs))]
    pub fn load(_cls: &PyType, path: String) -> PyResult<Self> {
        Ok(Self {
            dawg: dawg::Dawg::load(&path, CacheConfig::none()).expect("Failed to deserialize"),
        })
    }

    pub fn build(&mut self, text: Vec<u16>) {
        self.dawg.build(&text);
    }

    pub fn get_initial(&self) -> usize {
        self.dawg.get_initial().index()
    }

    pub fn transition(&self, state: usize, token: u16, use_failures: bool) -> Option<usize> {
        let state_index = NodeIndex::new(state);
        match self.dawg.transition(state_index, token, use_failures) {
            Some(q) => Some(q.index()),
            None => None,
        }
    }

    pub fn transition_and_count(
        &self,
        state: usize,
        token: u16,
        length: u64,
    ) -> (Option<usize>, u64) {
        let state_index = NodeIndex::new(state);
        let (new_state, new_length) = self.dawg.transition_and_count(state_index, token, length);
        match new_state {
            Some(q) => (Some(q.index()), new_length),
            None => (None, new_length),
        }
    }

    pub fn get_count(&self, state: usize) -> usize {
        let state_index = NodeIndex::new(state);
        self.dawg.get_node(state_index).get_count()
    }

    // Returns (State, TokenId)
    pub fn get_edges(&self, state: usize) -> Vec<(usize, u16)> {
        let state_index = NodeIndex::new(state);
        let graph = self.dawg.get_graph();
        graph
            .edges(state_index)
            .map(|edge| (edge.get_target().index(), edge.get_weight()))
            .collect()
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

    pub fn get_failure(&self, state: usize) -> Option<usize> {
        let state_node = NodeIndex::new(state);
        match self.dawg.get_node(state_node).get_failure() {
            Some(phi) => Some(phi.index()),
            None => None,
        }
    }

    pub fn get_length(&self, state: usize) -> u64 {
        let state_node = NodeIndex::new(state);
        self.dawg.get_node(state_node).get_length()
    }
}

impl DiskDawg {
    pub fn get_dawg(&self) -> &dawg::Dawg<u16, DefaultWeight, DefaultIx, Mb> {
        &self.dawg
    }
}
