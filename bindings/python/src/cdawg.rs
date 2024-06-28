use pyo3::prelude::*;

use std::cell::RefCell;
use std::rc::Rc;

use crate::cdawg_state::CdawgState;

use rusty_dawg::cdawg;
use rusty_dawg::graph::indexing::{DefaultIx, EdgeIndex, NodeIndex};
use rusty_dawg::graph::NodeRef;
use rusty_dawg::weight::DefaultWeight;

#[pyclass(unsendable)]
pub struct Cdawg {
    cdawg: cdawg::Cdawg<DefaultWeight, DefaultIx>,
}

// Wrap the normal Dawg class with a Python interface.
#[pymethods]
impl Cdawg {
    #[classattr]
    const EOS: u16 = u16::MAX;

    #[new]
    pub fn new(tokens: Vec<u16>) -> Self {
        let tokens_rc = Rc::new(RefCell::new(tokens));
        Self {
            cdawg: cdawg::Cdawg::new(tokens_rc),
        }
    }

    pub fn build(&mut self) {
        self.cdawg.build();
    }

    /// Build CDAWG incrementally. Use Rust build() at scale rather than calling through Python!
    pub fn update(&mut self, in_state: usize, start: usize, end: usize) -> (usize, usize) {
        let (new_state, new_start) = self.cdawg.update(NodeIndex::new(in_state), start, end);
        (new_state.index(), new_start)
    }

    pub fn fill_counts(&mut self) {
        let mut counter = cdawg::TopologicalCounter::new_ram();
        counter.fill_counts(&mut self.cdawg);
    }

    pub fn get_source(&self) -> usize {
        self.cdawg.get_source().index()
    }

    pub fn get_initial(&self) -> CdawgState {
        CdawgState {
            cs: self.cdawg.get_initial(),
        }
    }

    pub fn transition_and_count(&self, cs: CdawgState, token: u16) -> CdawgState {
        CdawgState {
            cs: self.cdawg.transition_and_count(cs.cs, token),
        }
    }

    pub fn get_edge_by_token(&self, state: usize, token: u16) -> Option<usize> {
        let node_idx = NodeIndex::new(state);
        let edge_idx = self.cdawg.get_edge_by_token(node_idx, token);
        match edge_idx {
            Some(e) => Some(e.index()),
            None => None,
        }
    }

    pub fn get_start_end_target(&self, edge_idx: usize) -> (usize, usize, usize) {
        let (start, end, target) = self.cdawg.get_start_end_target(EdgeIndex::new(edge_idx));
        // Adjust back to 0-indexed start for inference time.
        (start - 1, end, target.index())
    }

    pub fn get_count(&self, state: usize) -> usize {
        self.cdawg.get_count(NodeIndex::new(state))
    }

    /// gamma here is 0-indexed.
    pub fn implicitly_fail(&self, state: usize, gamma: (usize, usize)) -> CdawgState {
        CdawgState {
            cs: self.cdawg.implicitly_fail(NodeIndex::new(state), gamma),
        }
    }

    /// Return the length associated with a node.
    pub fn get_length(&self, state: usize) -> u64 {
        self.cdawg
            .get_graph()
            .get_node(NodeIndex::new(state))
            .get_length()
    }

    /// Get list of states that a state connects to. Useful for graph traversal.
    pub fn neighbors(&self, state: usize) -> Vec<usize> {
        let node = NodeIndex::new(state);
        self.cdawg.get_graph().neighbors(node).map(|x| x.index()).collect()
    }

    pub fn node_count(&self) -> usize {
        self.cdawg.node_count()
    }

    pub fn edge_count(&self) -> usize {
        self.cdawg.edge_count()
    }

    // Methods for inference time.

    pub fn get_suffix_count(&self, cs: CdawgState) -> usize {
        self.cdawg.get_suffix_count(cs.cs)
    }

    pub fn get_entropy(&self, cs: CdawgState) -> f64 {
        self.cdawg.get_entropy(cs.cs)
    }

    pub fn get_next_tokens(&self, cs: CdawgState) -> Vec<(u16, f64)> {
        self.cdawg.get_next_tokens(cs.cs)
    }
}
