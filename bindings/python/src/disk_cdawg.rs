use pyo3::prelude::*;
use pyo3::types::PyType;

use std::cell::RefCell;
use std::rc::Rc;

use crate::cdawg_state::CdawgState;

use rusty_dawg::cdawg;
use rusty_dawg::graph::indexing::{DefaultIx, EdgeIndex, NodeIndex};
use rusty_dawg::memory_backing::{CacheConfig, DiskBacking, DiskVec};
use rusty_dawg::weight::DefaultWeight;

type Mb = DiskBacking<DefaultWeight, (DefaultIx, DefaultIx), DefaultIx>;

#[pyclass(unsendable)]
pub struct DiskCdawg {
    cdawg: cdawg::Cdawg<DefaultWeight, DefaultIx, Mb>,
}

// Wrap the normal Dawg class with a Python interface.
#[pymethods]
impl DiskCdawg {
    #[classattr]
    const EOS: u16 = u16::MAX;

    // Assumes that tokens_path is a DiskVec already populated with the tokens we want to build on.
    #[new]
    pub fn new(tokens_path: String, mb_path: String, n_nodes: usize, n_edges: usize) -> Self {
        let tokens_vec = DiskVec::load(tokens_path).unwrap();
        let tokens_rc = Rc::new(RefCell::new(tokens_vec));
        let mb = DiskBacking::new(mb_path);
        let cache_config = CacheConfig::none();
        Self {
            cdawg: cdawg::Cdawg::with_capacity_mb(tokens_rc, mb, n_nodes, n_edges, cache_config),
        }
    }

    // Load a DiskCdawg that has already been built.
    #[classmethod]
    pub fn load(_cls: &PyType, tokens_path: String, mb_path: String) -> Self {
        let tokens_vec = DiskVec::load(tokens_path).unwrap();
        let tokens_rc = Rc::new(RefCell::new(tokens_vec));
        let cache_config = CacheConfig::none();
        Self {
            cdawg: cdawg::Cdawg::load(tokens_rc, mb_path, cache_config).unwrap(),
        }
    }

    pub fn build(&mut self) {
        self.cdawg.build();
    }

    pub fn fill_counts(&mut self, stack_path: String, capacity: usize) {
        let mut counter = cdawg::TopologicalCounter::new_disk(stack_path, capacity).unwrap();
        counter.fill_counts(&mut self.cdawg);
    }

    // TODO: Merge with above, adding default argument or TopologicalCounter object.
    pub fn fill_counts_ram(&mut self) {
        let mut counter = cdawg::TopologicalCounter::new_ram();
        counter.fill_counts(&mut self.cdawg);
    }

    /// Get list of arities for all nodes in CDAWG.
    pub fn traverse_arities(&mut self, capacity: usize) -> Vec<usize> {
        let mut traverser = cdawg::traverse_arity::TraverseArity::new_ram(capacity);
        traverser.traverse_arity(&mut self.cdawg)
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
