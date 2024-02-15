use pyo3::prelude::*;
use pyo3::types::PyType;

use std::cell::RefCell;
use std::rc::Rc;

use crate::cdawg_state::CdawgState;

use rusty_dawg::cdawg;
use rusty_dawg::graph::indexing::{EdgeIndex, NodeIndex, DefaultIx};
use rusty_dawg::weight::DefaultWeight;
use rusty_dawg::memory_backing::DiskBacking;
use rusty_dawg::memory_backing::disk_backing::disk_vec::DiskVec;
use rusty_dawg::cdawg::cdawg_edge_weight::CdawgEdgeWeight;

type Mb = DiskBacking<DefaultWeight, CdawgEdgeWeight<DefaultIx>, DefaultIx>;

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
        Self {
            cdawg: cdawg::Cdawg::with_capacity_mb(tokens_rc, mb, n_nodes, n_edges),
        }
    }

    // Load a DiskCdawg that has already been built.
    #[classmethod]
    pub fn load(_cls: &PyType, tokens_path: String, mb_path: String) -> Self {
        let tokens_vec = DiskVec::load(tokens_path).unwrap();
        let tokens_rc = Rc::new(RefCell::new(tokens_vec));
        Self {
            cdawg: cdawg::Cdawg::load(tokens_rc, mb_path).unwrap(),
        }
    }

    pub fn build(&mut self) {
        self.cdawg.build();
    }

    pub fn fill_counts(&mut self, stack_path: String, capacity: usize) {
        let mut counter = cdawg::TopologicalCounter::new_disk(stack_path, capacity).unwrap();
        counter.fill_counts(&mut self.cdawg);
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
}
