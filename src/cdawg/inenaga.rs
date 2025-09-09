// Follows the algorithm from "On-line construction of compact directed acyclic word graphs"
//
// Paper: https://www.sciencedirect.com/science/article/pii/S0166218X04003464
// Python: https://github.com/raedwulf/PyCDAWG/blob/master/cdawg.py
//
// # Notes on indexing
// The pseudo-code uses q, (k, p - 1) to represent an active point (see Figure 12).
// There are two differences with indexing in our implementation from the pseudo-code:
//   1) Ours is 0-indexed (pseudo-code is 1-indexed).
//   2) Our spans have exclusive ends (p - 1 is inclusive).
// As a consequence:
//  * k = our start + 1
//  * p - 1 = our end
//  * their length = our length - 1
// Note that 1 is already subtracted from p in many subroutines.
//
// # Notes on representing states
// While building, astate is an Option<NodeIndex>. None means the failure of the initial state.
//
// # Notes on counts
// This code sets the count of each sink node to 1 and every other node to 0. This is because the
// counts can only be computed efficiently after building has finished, and this is the format that
// the second step expects.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::convert::TryInto;
use std::path::Path;
use std::rc::Rc;

use crate::cdawg::cdawg_state::CdawgState;
use crate::cdawg::comparator::CdawgComparator;
use crate::cdawg::metadata::CdawgMetadata;
use crate::cdawg::token_backing::TokenBacking;
use crate::graph::avl_graph::edge::EdgeMutRef;
use crate::graph::avl_graph::node::NodeMutRef;
use crate::graph::avl_graph::AvlGraph;
use crate::graph::indexing::{DefaultIx, EdgeIndex, IndexType, NodeIndex};
use crate::graph::{EdgeRef, NodeRef};
use crate::memory_backing::{CacheConfig, DiskBacking, MemoryBacking, RamBacking};
use crate::weight::{DefaultWeight, Weight};

// TODO: Add TokenBacking for tokens

pub struct Cdawg<W = DefaultWeight, Ix = DefaultIx, Mb = RamBacking<W, (Ix, Ix), Ix>>
where
    Ix: IndexType,
    W: Weight + Clone,
    Mb: MemoryBacking<W, (Ix, Ix), Ix>,
{
    tokens: Rc<RefCell<dyn TokenBacking<u16>>>,
    graph: AvlGraph<W, (Ix, Ix), Ix, Mb>,
    source: NodeIndex<Ix>,
    sink: NodeIndex<Ix>,
    end_position: usize, // End position of current document.
}

impl<W, Ix> Cdawg<W, Ix>
where
    Ix: IndexType,
    W: Weight + Serialize + for<'de> Deserialize<'de> + Clone,
{
    pub fn new(tokens: Rc<RefCell<dyn TokenBacking<u16>>>) -> Self {
        let mb: RamBacking<W, (Ix, Ix), Ix> = RamBacking::default();
        Self::new_mb(tokens, mb)
    }
}

impl<W, Ix> Cdawg<W, Ix, DiskBacking<W, (Ix, Ix), Ix>>
where
    Ix: IndexType + Serialize + for<'de> serde::Deserialize<'de>,
    W: Weight + Copy + Serialize + for<'de> Deserialize<'de> + Clone + Default,
    (Ix, Ix): Serialize + for<'de> Deserialize<'de>,
{
    pub fn load<P: AsRef<Path> + Clone + std::fmt::Debug>(
        tokens: Rc<RefCell<dyn TokenBacking<u16>>>,
        path: P,
        cache_config: CacheConfig,
    ) -> Result<Self> {
        // Load source/sink from config file if it exists.
        let path2 = path.clone();
        let graph = AvlGraph::load(path, cache_config)?;

        let mut config_path = path2.as_ref().to_path_buf();
        config_path.push("metadata.json");
        if config_path.exists() {
            // FIXME(#98): This will fail silently if config file exists but is empty.
            let config = CdawgMetadata::load_json(config_path)?;
            Ok(Self {
                tokens,
                graph,
                source: NodeIndex::new(config.source),
                sink: NodeIndex::new(config.sink),
                end_position: config.end_position,
            })
        } else {
            Ok(Self {
                tokens,
                graph,
                source: NodeIndex::new(0),
                sink: NodeIndex::new(1),
                end_position: 0,
            })
        }
    }
}

impl<W, Ix, Mb> Cdawg<W, Ix, Mb>
where
    Ix: IndexType,
    W: Weight + Serialize + for<'de> Deserialize<'de> + Clone,
    Mb: MemoryBacking<W, (Ix, Ix), Ix>,
    Mb::EdgeRef: Copy,
{
    pub fn new_mb(tokens: Rc<RefCell<dyn TokenBacking<u16>>>, mb: Mb) -> Cdawg<W, Ix, Mb> {
        let mut graph: AvlGraph<W, (Ix, Ix), Ix, Mb> = AvlGraph::new_mb(mb);
        let source = graph.add_node(W::new(0, None, 0));
        // FIXME: Hacky type conversion for sink failure.
        let sink = graph.add_node(W::new(0, Some(NodeIndex::new(source.index())), 1));
        Self {
            tokens,
            graph,
            source,
            sink,
            end_position: 0,
        }
    }

    pub fn with_capacity_mb(
        tokens: Rc<RefCell<dyn TokenBacking<u16>>>,
        mb: Mb,
        n_nodes: usize,
        n_edges: usize,
        cache_config: CacheConfig,
    ) -> Cdawg<W, Ix, Mb> {
        let mut graph: AvlGraph<W, (Ix, Ix), Ix, Mb> =
            AvlGraph::with_capacity_mb(mb, n_nodes, n_edges, cache_config);
        let source = graph.add_node(W::new(0, None, 0));
        // FIXME: Hacky type conversion for sink failure.
        let sink = graph.add_node(W::new(0, Some(NodeIndex::new(source.index())), 1));
        Self {
            tokens,
            graph,
            source,
            sink,
            end_position: 0,
        }
    }

    // Tokens needs to be fully populated and contain end-of-document tokens for this to work.
    pub fn build(&mut self) {
        let (mut state, mut start) = (self.source, 1);
        let length = self.tokens.borrow().len();
        for idx in 1..length + 1 {
            (state, start) = self.update(state, start, idx);
            if self.tokens.borrow().get(idx - 1) == u16::MAX {
                (state, start) = self.end_document(idx, idx);
            }
        }
    }

    pub fn update(
        &mut self,
        in_state: NodeIndex<Ix>, // Cannot be null.
        mut start: usize,
        end: usize,
    ) -> (NodeIndex<Ix>, usize) {
        // Update self.e, which is also the length of the current sink.
        self.end_position += 1;
        let sink_length = self.graph.get_node(self.sink).get_length();
        self.graph
            .get_node_mut(self.sink)
            .set_length(sink_length + 1);

        let mut dest: Option<NodeIndex<Ix>> = None;
        let mut r = NodeIndex::end();
        let mut opt_state: Option<NodeIndex<Ix>> = Some(in_state);
        let mut opt_old_r: Option<NodeIndex<Ix>> = None;
        let token = self.tokens.borrow().get(end - 1); // Map p back to 0-indexing
        while !self.check_end_point(opt_state, (start, end - 1), token) {
            // Within the loop, never possible for opt_state to be null.
            let state = opt_state.unwrap();

            if start < end {
                // Implicit case checks when an edge is active.
                let cur_dest = self.extension(state, (start, end - 1));
                if dest == Some(cur_dest) {
                    // This call updates the count appropriately.
                    self.redirect_edge(state, (start, end - 1), r);
                    let fstate = self.graph.get_node(state).get_failure();
                    (opt_state, start) = self.canonize(fstate, (start, end - 1));
                    continue;
                } else {
                    dest = Some(cur_dest);
                    r = self.split_edge(state, (start, end - 1));
                }
            } else {
                // Explicit case checks when a state is active.
                r = state;
            }

            // 1) Add a new OPEN edge from r to sink (that can grow via pointer).
            // Should work correctly when tokens[end - 1] is u16::MAX.
            self.add_balanced_edge(r, self.sink, (end, Ix::max_value().index()));

            // 2) Set failure transition.
            if let Some(old_r) = opt_old_r {
                self.graph.get_node_mut(old_r).set_failure(Some(r));
            }
            opt_old_r = Some(r);

            // 3) Update state by canonizing the fstate.
            let fstate = self.graph.get_node(state).get_failure();
            (opt_state, start) = self.canonize(fstate, (start, end - 1));
        }

        if let Some(old_r) = opt_old_r {
            self.graph.get_node_mut(old_r).set_failure(opt_state);
        }
        self.separate_node(opt_state, (start, end))
    }

    // Update the document_id, add a sink node, and return new (state, start).
    pub fn end_document(
        &mut self,
        idx: usize,    // Index of end-of-document token in CURRENT document.
        doc_id: usize, // Canonical doc ID for CURRENT document.
    ) -> (NodeIndex<Ix>, usize) {
        // Add a looped edge on the sink node encoding
        // We do this after the document is done so it never gets cloned.
        // At this point, idx == self.end_position.
        let weight = (idx, doc_id); // doc_id is basically a label for node
        self.add_balanced_edge(self.sink, self.sink, weight);

        let source = NodeIndex::new(self.source.index());
        self.sink = self.graph.add_node(W::new(0, Some(source), 1));
        (self.source, idx + 1)
    }

    // This is just following a transition (doesn't eat up everything potentially)
    // Note: 1-indexed!
    fn extension(&self, state: NodeIndex<Ix>, gamma: (usize, usize)) -> NodeIndex<Ix> {
        let (start, end) = gamma;
        if start > end {
            return state;
        }
        // let token = self.tokens.borrow().get(start - 1);
        // let e = self.get_edge_by_token(state, token);
        let e = self.get_edge_by_token_index(state, start - 1);
        self.graph.get_edge(e.unwrap()).get_target()
    }

    // Change the target of the edge coming out of state with path gamma.
    // Updates the counts to reflect the new graph structure.
    // Note: 1-indexed!
    pub fn redirect_edge(
        &mut self,
        state: NodeIndex<Ix>,
        gamma: (usize, usize),
        target: NodeIndex<Ix>,
    ) {
        let (start, end) = gamma;
        // let token = self.tokens.borrow().get(start - 1);
        // let edge_idx = self.get_edge_by_token(state, token).unwrap();
        let edge_idx = self.get_edge_by_token_index(state, start - 1).unwrap();
        let edge_ref = self.graph.get_edge(edge_idx);
        let old_target = edge_ref.get_target();
        let (found_start, _) = self.get_span(edge_ref.get_weight(), old_target);

        // Doesn't actually use graph.reroute_edge
        let weight = self._new_edge_weight(found_start, found_start + end - start);
        self.graph.get_edge_mut(edge_idx).set_weight(weight);
        self.graph.get_edge_mut(edge_idx).set_target(target);
    }

    // Split the edge, copying counts, and leave failure transitions unedited.
    fn split_edge(&mut self, q: NodeIndex<Ix>, gamma: (usize, usize)) -> NodeIndex<Ix> {
        // First, create a new node and set it's length.
        let v = self.graph.add_node(self.graph.get_node(q).get_weight());
        let q_length = self.graph.get_node(q).get_length();
        let gamma_length = <usize as TryInto<u64>>::try_into(gamma.1 - gamma.0 + 1).unwrap();
        self.graph
            .get_node_mut(v)
            .set_length(q_length + gamma_length);
        self.graph.get_node_mut(v).set_count(0); // 0 for non-sink node.

        // Next, get the existing edge we're going to split.
        // let token = self.tokens.borrow().get(gamma.0 - 1); // 0-indexed
        // let edge_idx = self.get_edge_by_token(q, token);
        let edge_idx = self.get_edge_by_token_index(q, gamma.0 - 1);
        let edge = self.graph.get_edge(edge_idx.unwrap());
        let (mut start, end) = (edge.get_weight().0.index(), edge.get_weight().1.index()); // We DON'T call get_span because we want to keep end pointers.
        start += 1; // Map back to Inenaga 1-indexed!
        let target = edge.get_target();

        // Reroute this edge into v.
        self.graph
            .get_edge_mut(edge_idx.unwrap())
            .set_weight(self._new_edge_weight(start, start + gamma.1 - gamma.0));
        self.graph.get_edge_mut(edge_idx.unwrap()).set_target(v);

        // Create a new edge from v to the original target.
        self.add_balanced_edge(v, target, (start + gamma.1 - gamma.0 + 1, end));

        v
    }

    fn separate_node(
        &mut self,
        mut state: Option<NodeIndex<Ix>>,
        gamma: (usize, usize),
    ) -> (NodeIndex<Ix>, usize) {
        let (mut start, end) = gamma;
        let (opt_state1, start1) = self.canonize(state, (start, end));
        let state1 = opt_state1.unwrap();

        // Implicit case: active point is along an edge.
        if start1 <= end {
            return (state1, start1);
        }

        let length = match state {
            Some(q) => self.graph.get_node(q).get_length() as i64,
            None => -1,
        };
        let length1 = self.graph.get_node(state1).get_length() as i64;
        if length1 == length + (end - start + 1) as i64 {
            return (state1, start1);
        }

        // Non-solid, explicit case: clone node and set its length
        let mut weight = self.graph.get_node(state1).get_weight().clone();
        weight.set_length((length + (end - start + 1) as i64) as u64);
        let new_state = self.graph.add_node(weight);
        self.graph.get_node_mut(new_state).set_count(0); // 0 for non-sink.
        self.graph.clone_edges(state1, new_state);

        // Update the failure transitions.
        self.graph
            .get_node_mut(new_state)
            .set_failure(self.graph.get_node(state1).get_failure());
        self.graph.get_node_mut(state1).set_failure(Some(new_state));

        // Replace edges from state to state1 with edges to new_state.
        // We know that state is non-null here.
        loop {
            // Reroute tokens[start-1] edge to new_state via (start, end)
            // let token = self.tokens.borrow().get(start - 1);
            // let edge_idx = self.get_edge_by_token(state.unwrap(), token);
            let edge_idx = self.get_edge_by_token_index(state.unwrap(), start - 1);
            self.graph
                .get_edge_mut(edge_idx.unwrap())
                .set_weight(self._new_edge_weight(start, end));
            self.graph
                .get_edge_mut(edge_idx.unwrap())
                .set_target(new_state);

            let fstate = self.graph.get_node(state.unwrap()).get_failure();
            (state, start) = self.canonize(fstate, (start, end - 1));
            if (opt_state1, start1) != self.canonize(state, (start, end)) {
                break;
            }
        }
        (new_state, end + 1)
    }

    // The point of this is to move through the DAWG until state uses up as much as possible/start is maximized.
    // Returns the last state passed through, and the beginning of the active edge out of it.
    // Assumes 1 indexing!! This is because gamma would have to encode negative values with 0 indexing.
    fn canonize(
        &self,
        mut state: Option<NodeIndex<Ix>>,
        gamma: (usize, usize),
    ) -> (Option<NodeIndex<Ix>>, usize) {
        let (mut start, end) = gamma;
        if start > end {
            // Means we are at a state.
            return (state, start);
        }

        let mut found_start: usize;
        let mut found_end: usize;
        let mut found_state: NodeIndex<Ix>;
        match state {
            Some(q) => {
                // let token = self.tokens.borrow().get(start - 1);
                // let edge_idx = self.get_edge_by_token(q, token).unwrap();
                let edge_idx = self.get_edge_by_token_index(q, start - 1).unwrap();
                (found_start, found_end, found_state) = self.get_start_end_target(edge_idx);
            }
            None => {
                (found_start, found_end, found_state) = (0, 0, self.source);
            }
        }

        while found_end + start <= end + found_start {
            // Written this way to avoid overflow.
            start += found_end + 1 - found_start; // Written this way to avoid overflow.
            state = Some(found_state);
            if start <= end {
                // let token = self.tokens.borrow().get(start - 1);
                // let edge_idx = self.get_edge_by_token(found_state, token).unwrap();
                let edge_idx = self
                    .get_edge_by_token_index(found_state, start - 1)
                    .unwrap();
                (found_start, found_end, found_state) = self.get_start_end_target(edge_idx);
            }
        }
        (state, start)
    }

    // Return true if we can keep going; false otherwise.
    // 1-indexed!
    fn check_end_point(
        &self,
        state: Option<NodeIndex<Ix>>,
        gamma: (usize, usize),
        token: u16,
    ) -> bool {
        let (start, end) = gamma;
        if start <= end {
            // let wk = self.tokens.borrow().get(start - 1);
            // let e = self.get_edge_by_token(state.unwrap(), wk).unwrap();
            let e = self
                .get_edge_by_token_index(state.unwrap(), start - 1)
                .unwrap();
            let edge = self.graph.get_edge(e);
            let (found_start, found_end) = self.get_span(edge.get_weight(), edge.get_target());
            if found_end - found_start < end - start {
                return false;
            }

            // No +1 because 0-indexed.
            let existing_token = self.tokens.borrow().get(found_start + end - start);
            if token != u16::MAX || existing_token != u16::MAX {
                token == existing_token
            } else {
                // Compare based on whether these are the same end-of-text tokens.
                end == found_start + end - start
            }
        } else {
            match state {
                Some(phi) => {
                    // token == tokens[end]
                    // let edge_idx = self.get_edge_by_token(phi, token);
                    let edge_idx = if token != u16::MAX {
                        self.get_edge_by_token(phi, token)
                    } else {
                        self.get_edge_by_token_index(phi, end)
                    };

                    edge_idx.is_some()
                }
                None => true,
            }
        }
    }

    // These helper methods are useful.
    // Could possible deprecate or merge these together.

    // Add a new edge with appropriate weight.
    // Take in indices with paper's conventions and return ours.
    // Maybe make this a macro?
    fn _new_edge_weight(&mut self, start: usize, end: usize) -> (Ix, Ix) {
        (
            Ix::new(start - 1), // Map start to 0-indexed
            Ix::new(end),
        ) // Keep end as 1-indexed
    }

    // Get the Inenaga-indexed span associated with an edge.
    // Maybe make this a macro?
    fn get_span(&self, weight: (Ix, Ix), target: NodeIndex<Ix>) -> (usize, usize) {
        let (start, end) = (weight.0.index(), weight.1.index());
        // Shift to 1-indexed and retrieve value of end pointer.
        if end < Ix::max_value().index() {
            (start + 1, end)
        } else {
            // If there is a self-loop, we are at a different document.
            let edge_idx = self.graph.get_node(target).get_first_edge();
            if edge_idx == EdgeIndex::end() {
                // We are in the active document.
                (start + 1, self.end_position)
            } else {
                // We are at the sink for a different document.
                let weight = self.graph.get_edge(edge_idx).get_weight();
                let (e, _) = (weight.0.index(), weight.1.index());
                (start + 1, e + 1) // Adjust both to be 1-indexed.
            }
            // let e = self.graph.get_node(target).get_length();
            // (start + 1, e.try_into().unwrap())
        }
    }

    // Get start, end, target associated with an edge.
    // This is 1-indexed for legacy reasons!
    pub fn get_start_end_target(&self, edge_idx: EdgeIndex<Ix>) -> (usize, usize, NodeIndex<Ix>) {
        let edge_ref = self.graph.get_edge(edge_idx);
        let target = edge_ref.get_target();
        let span = self.get_span(edge_ref.get_weight(), target);
        // Shift to 1-indexed and retrieve value of end pointer.
        (span.0, span.1, target)
    }

    // Convenience methods.

    pub fn get_graph(&self) -> &AvlGraph<W, (Ix, Ix), Ix, Mb> {
        &self.graph
    }

    pub fn get_source(&self) -> NodeIndex<Ix> {
        self.source
    }

    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }

    pub fn balance_ratio(&self, n_states: usize) -> f64 {
        let mut max_ratio = 1.;
        for _state in 0..n_states {
            let ratio = self.graph.balance_ratio(self.get_source());
            if ratio > max_ratio {
                max_ratio = ratio;
            }
        }
        max_ratio
    }

    // Only well-defined when token is not end-of-text.
    pub fn get_edge_by_token(&self, state: NodeIndex<Ix>, token: u16) -> Option<EdgeIndex<Ix>> {
        if token != u16::MAX {
            let weight = (Ix::new(0), Ix::new(0)); // Doesn't matter.
            let cmp = CdawgComparator::new_with_token(self.tokens.clone(), token);
            self.graph
                .get_edge_by_weight_cmp(state, weight, Box::new(cmp))
        } else {
            None
        }
    }

    // Handle end-of-text tokens correctly.
    pub fn get_edge_by_token_index(
        &self,
        state: NodeIndex<Ix>,
        token_idx: usize,
    ) -> Option<EdgeIndex<Ix>> {
        let weight = (Ix::new(token_idx), Ix::new(token_idx + 1));
        let token = self.tokens.borrow().get(token_idx);
        let cmp = CdawgComparator::new_with_token(self.tokens.clone(), token);
        self.graph
            .get_edge_by_weight_cmp(state, weight, Box::new(cmp))
    }

    pub fn add_balanced_edge(
        &mut self,
        state: NodeIndex<Ix>,
        target: NodeIndex<Ix>,
        gamma: (usize, usize),
    ) {
        // We should have gamma.0 <= gamma.1
        let weight = self._new_edge_weight(gamma.0, gamma.1);
        let token = self.tokens.borrow().get(gamma.0 - 1); // Map to 0-indexed
        let cmp = CdawgComparator::new_with_token(self.tokens.clone(), token);
        self.graph
            .add_balanced_edge_cmp(state, target, weight, Box::new(cmp))
    }

    // Methods for inference with the CDAWG.

    // Get the source state and initial values for transition quantities.
    pub fn get_initial(&self) -> CdawgState<Ix> {
        CdawgState {
            state: self.source,
            edge_start: 0,
            start: 0,
            end: 0,
            target: Some(self.source),
            length: 0,
        }
    }

    // Transition and track length analogously to the DAWG.
    pub fn transition_and_count(&self, mut cs: CdawgState<Ix>, token: u16) -> CdawgState<Ix> {
        if cs.target.is_none() {
            // Corresponds to the case where we are in the null state after failing.
            self.get_initial()
        } else if cs.start == cs.end {
            // We are at a state. Analogous to DAWG case.
            let e = self.get_edge_by_token(cs.target.unwrap(), token);
            if let Some(e_val) = e {
                let edge = self.graph.get_edge(e_val);
                let gamma = self.get_span(edge.get_weight(), edge.get_target());
                return CdawgState {
                    state: cs.target.unwrap(),
                    edge_start: gamma.0 - 1, // -1 for 0-indexing
                    start: gamma.0,          // -1 for 0-indexing, +1 to increment
                    end: gamma.1,
                    target: Some(edge.get_target()),
                    length: cs.length + 1,
                };
            }
            let fail_cs = self.implicitly_fail(cs.target.unwrap(), (cs.end, cs.end));
            self.transition_and_count(fail_cs, token)
        } else {
            // We are on an edge.
            let cur_token = self.tokens.borrow().get(cs.start);
            if token == cur_token {
                cs.start += 1;
                cs.length += 1;
                return cs;
            }
            let fail_cs = self.implicitly_fail(cs.state, (cs.edge_start, cs.start));
            self.transition_and_count(fail_cs, token)
        }
    }

    // Inference-time version of canonize. Crucially:
    //   1. returns target state.
    fn inference_canonize(
        &self,
        mut state: Option<NodeIndex<Ix>>,
        gamma: (usize, usize),
    ) -> (
        Option<NodeIndex<Ix>>,
        usize,
        Option<NodeIndex<Ix>>,
        usize,
        usize,
    ) {
        let (mut start, end) = gamma;
        if start > end {
            // Means we are at a state.
            return (state, start, state, start, end);
        }

        let mut found_start: usize;
        let mut found_end: usize;
        let mut found_state: NodeIndex<Ix>;
        match state {
            Some(q) => {
                let token = self.tokens.borrow().get(start - 1);
                let edge_idx = self.get_edge_by_token(q, token).unwrap();
                (found_start, found_end, found_state) = self.get_start_end_target(edge_idx);
            }
            None => {
                // Changed these to (1, 1) to avoid subtraction overflow issue.
                (found_start, found_end, found_state) = (1, 1, self.source);
            }
        }

        while found_end + start <= end + found_start {
            // Written this way to avoid overflow.
            start += found_end + 1 - found_start; // Written this way to avoid overflow.
            state = Some(found_state);
            if start <= end {
                let token = self.tokens.borrow().get(start - 1);
                let edge_idx = self.get_edge_by_token(found_state, token).unwrap();
                (found_start, found_end, found_state) = self.get_start_end_target(edge_idx);
            }
        }
        // Map found_start to 1-indexed when we return it.
        (state, start, Some(found_state), found_start, found_end)
    }

    // Generalizes failure transition for when we have state + gamma.
    // This is 0-indexed since we use it at inference time.
    // Gamma represents a path of tokens we want to follow from fstate.
    pub fn implicitly_fail(&self, state: NodeIndex<Ix>, gamma: (usize, usize)) -> CdawgState<Ix> {
        let (start, end) = gamma;
        let fstate = self.graph.get_node(state).get_failure();

        // Is it cleaner to just rewrite this manually?
        let (opt_state, mut new_start, opt_target, mut found_start, found_end) =
            self.inference_canonize(fstate, (start + 1, end));
        new_start -= 1;
        found_start -= 1;
        match opt_state {
            Some(q) => {
                // Canonize has gotten to a state.
                if new_start == end {
                    CdawgState {
                        state: q,
                        edge_start: found_start,
                        start: found_end,
                        end: found_end,
                        target: opt_state,
                        length: self.graph.get_node(q).get_length(),
                    }
                } else {
                    let progress = end - new_start;
                    CdawgState {
                        state: q,
                        edge_start: found_start,
                        start: found_start + progress,
                        end: found_end,
                        target: opt_target,
                        // FIXME: Why do we potentially get overflow here?
                        length: self.graph.get_node(q).get_length() + progress as u64,
                    }
                }
            }
            // We failed from initial state.
            None => CdawgState {
                state: self.source,
                edge_start: 0,
                start: 0,
                end: 0,
                target: None,
                length: 0, // Actually -1 but unsigned.
            },
        }
    }

    pub fn get_count(&self, state: NodeIndex<Ix>) -> usize {
        self.graph.get_node(state).get_count()
    }

    pub fn set_count(&mut self, state: NodeIndex<Ix>, count: usize) {
        self.graph.get_node_mut(state).set_count(count);
    }

    ///Save metadata
    pub fn save_metadata<P: AsRef<Path> + Clone>(&self, path: P) -> Result<()> {
        let mut config_path = path.as_ref().to_path_buf();
        config_path.push("metadata.json");
        let config = CdawgMetadata {
            source: self.source.index(),
            sink: self.sink.index(),
            end_position: self.end_position,
        };
        config.save_json(config_path)
    }

    // TODO(#100): Refactor these into an Infinigram class that wraps a Cdawg

    /// Get the count of the suffix matched by a CdawgState.
    pub fn get_suffix_count(&self, cs: CdawgState<Ix>) -> usize {
        self.get_count(cs.target.unwrap())
    }

    /// Get the entropy of a CDAWG state in bits.
    pub fn get_entropy(&self, cs: CdawgState<Ix>) -> f64 {
        let (state, gamma) = cs.get_state_and_gamma();
        if gamma.0 != gamma.1 {
            return 0.;
        }

        let q = state.unwrap();
        let denom = self.get_count(q);
        let mut sum = 0.;
        for next_state in self.get_graph().neighbors(q) {
            let prob = (self.get_count(next_state) as f64) / (denom as f64);
            sum -= prob * f64::log2(prob);
        }
        sum
    }

    pub fn get_next_tokens(&self, cs: CdawgState<Ix>) -> Vec<(u16, f64)> {
        let (state, gamma) = cs.get_state_and_gamma();
        if gamma.0 != gamma.1 {
            let token = self.tokens.borrow().get(gamma.1);
            return vec![(token, 1.)];
        }

        let q = state.unwrap();
        let denom = self.get_count(q);
        let mut tokens = Vec::new();
        for edge in self.get_graph().edges(q) {
            // let edge_ref = self.graph.get_edge(edge_idx);
            let next_state = edge.get_target();
            let span = self.get_span(edge.get_weight(), next_state);
            let token = self.tokens.borrow().get(span.0 - 1); // Shift to 0 indexing.
            let prob = (self.get_count(next_state) as f64) / (denom as f64);
            tokens.push((token, prob));
        }
        tokens
    }
}

#[cfg(test)]
#[allow(unused_variables)]
#[allow(unused_imports)]
#[allow(unused_assignments)]
mod tests {
    use super::*;
    use crate::cdawg::TopologicalCounter;
    use crate::memory_backing::DiskVec;
    use tempfile::tempdir;

    macro_rules! get_edge {
        // `()` indicates that the macro takes no argument.
        ($c:expr, $q:expr, $w:expr) => {
            $c.graph.get_edge($c.get_edge_by_token($q, $w).unwrap())
        };
    }

    macro_rules! get_span {
        ($c:expr, $e:expr) => {
            $c.get_span($e.get_weight(), $e.get_target())
        };
    }

    type Span = (usize, usize);

    fn to_inenaga(gamma: Span) -> Span {
        (gamma.0 + 1, gamma.1)
    }

    #[test]
    fn test_canonize() {
        // Test canonize, which uses 1-indexing!
        let mut cdawg: Cdawg = Cdawg::new(Rc::new(RefCell::new(vec![0, 1, 2])));
        let q = cdawg
            .graph
            .add_node(DefaultWeight::new(1, Some(cdawg.source), 0));
        cdawg.add_balanced_edge(cdawg.source, q, (1, 1));
        cdawg.add_balanced_edge(q, cdawg.sink, (2, 3));

        let (mut state, mut start) = cdawg.canonize(None, (1, 1));
        assert_eq!(state.unwrap().index(), cdawg.source.index());
        assert_eq!(start, 2);

        (state, start) = cdawg.canonize(None, (1, 0));
        assert_eq!(state, None);
        assert_eq!(start, 1);

        (state, start) = cdawg.canonize(Some(cdawg.source), (1, 0));
        assert_eq!(state.unwrap().index(), cdawg.source.index());
        assert_eq!(start, 1);

        (state, start) = cdawg.canonize(Some(cdawg.source), (1, 1));
        assert_eq!(state.unwrap().index(), q.index());
        assert_eq!(start, 2); // No tokens remaining.

        (state, start) = cdawg.canonize(Some(cdawg.source), (1, 2));
        assert_eq!(state.unwrap().index(), q.index());
        assert_eq!(start, 2); // One token remaining.

        (state, start) = cdawg.canonize(Some(cdawg.source), (1, 3));
        assert_eq!(state.unwrap().index(), cdawg.sink.index());
        assert_eq!(start, 4); // No tokens remaining.

        (state, start) = cdawg.canonize(Some(q), (2, 2));
        assert_eq!(state.unwrap().index(), q.index());
        assert_eq!(start, 2); // One token remaining.

        (state, start) = cdawg.canonize(Some(q), (2, 3));
        assert_eq!(state.unwrap().index(), cdawg.sink.index());
        assert_eq!(start, 4); // No tokens remaining.
    }

    #[test]
    fn test_check_end_point() {
        // This is 1-indexed!
        let mut cdawg: Cdawg =
            Cdawg::new(Rc::new(RefCell::new(vec![0, 1, 2, u16::MAX, 2, u16::MAX])));
        cdawg.add_balanced_edge(cdawg.source, cdawg.sink, (1, 4));
        assert!(cdawg.check_end_point(Some(cdawg.source), to_inenaga((0, 0)), 0));
        assert!(!cdawg.check_end_point(Some(cdawg.source), to_inenaga((0, 0)), 1));
        assert!(cdawg.check_end_point(Some(cdawg.source), to_inenaga((0, 1)), 1));
        assert!(!cdawg.check_end_point(Some(cdawg.source), to_inenaga((0, 1)), 0));

        // From the null state, anything should return true.
        assert!(cdawg.check_end_point(None, to_inenaga((0, 0)), 0));
        assert!(cdawg.check_end_point(None, to_inenaga((0, 0)), 1));
        assert!(cdawg.check_end_point(None, to_inenaga((0, 0)), 2));

        // Check with end-of-text tokens on edge.
        assert!(cdawg.check_end_point(Some(cdawg.source), (1, 3), u16::MAX));
        // Has not been inserted yet.
        assert!(!cdawg.check_end_point(Some(cdawg.source), (1, 5), u16::MAX));

        // Check with end-of-text tokens at state.
        // here, end must be the token index (when 0-indexed)
        cdawg.add_balanced_edge(cdawg.source, cdawg.sink, (4, 4));
        assert!(cdawg.check_end_point(Some(cdawg.source), (4, 3), u16::MAX));
        assert!(!cdawg.check_end_point(Some(cdawg.source), (6, 5), u16::MAX));
    }

    #[test]
    fn test_extension() {
        let mut cdawg: Cdawg = Cdawg::new(Rc::new(RefCell::new(vec![0, 1, 2])));
        cdawg.add_balanced_edge(cdawg.source, cdawg.sink, (1, 3));
        let target = cdawg.extension(cdawg.source, to_inenaga((0, 3)));
        assert_eq!(target.index(), cdawg.sink.index());
    }

    #[test]
    fn test_split_edge() {
        let mut cdawg: Cdawg = Cdawg::new(Rc::new(RefCell::new(vec![0, 1, 2])));

        cdawg.add_balanced_edge(cdawg.source, cdawg.sink, (1, 3));
        let v = cdawg.split_edge(cdawg.source, to_inenaga((0, 1)));

        let idx1 = cdawg.graph.get_node(cdawg.source).get_first_edge();
        let edge1 = cdawg.graph.get_edge(idx1);
        assert_eq!(edge1.get_target().index(), v.index());
        assert_eq!(edge1.get_weight().0.index(), 0);
        assert_eq!(edge1.get_weight().1.index(), 1);

        let idx2 = cdawg.graph.get_node(v).get_first_edge();
        let edge2 = cdawg.graph.get_edge(idx2);
        assert_eq!(edge2.get_target().index(), cdawg.sink.index());
        assert_eq!(edge2.get_weight().0.index(), 1);
        assert_eq!(edge2.get_weight().1.index(), 3);

        let target1 = cdawg.extension(cdawg.source, to_inenaga((0, 1)));
        assert_eq!(target1.index(), v.index());
        let target2 = cdawg.extension(target1, to_inenaga((1, 3)));
        assert_eq!(target2.index(), cdawg.sink.index());
    }

    #[test]
    fn test_redirect_edge() {
        let mut cdawg: Cdawg = Cdawg::new(Rc::new(RefCell::new(vec![0, 1, 2])));
        cdawg.add_balanced_edge(cdawg.source, cdawg.sink, (1, 3));
        let target = cdawg.graph.add_node(DefaultWeight::new(0, None, 2));
        cdawg.redirect_edge(cdawg.source, to_inenaga((0, 2)), target); // Arguments are 1-indexed

        let idx = cdawg.graph.get_node(cdawg.source).get_first_edge();
        let edge: *const crate::graph::avl_graph::Edge<(DefaultIx, DefaultIx)> =
            cdawg.graph.get_edge(idx);
        assert_eq!(edge.get_target().index(), target.index());
        // Graph is 0-indexed
        assert_eq!(edge.get_weight().0.index(), 0);
        assert_eq!(edge.get_weight().1.index(), 2);
    }

    #[test]
    fn test_separate_node_null() {
        let mut cdawg: Cdawg = Cdawg::new(Rc::new(RefCell::new(vec![0, 1, 2])));
        let c = cdawg
            .graph
            .add_node(DefaultWeight::new(1, Some(cdawg.source), 0));
        cdawg.add_balanced_edge(cdawg.source, c, (1, 1));

        // First step of cocoa should go back to initial state.
        let (state, start) = cdawg.separate_node(None, (1, 1));
        assert_eq!(state, cdawg.source);
        assert_eq!(start, 2);
    }

    #[test]
    fn test_update_cocoao() {
        // Minimal example from Figure 16 that invokes separate_node.
        let c = 0;
        let o = 1;
        let a = 2;
        let mut cdawg: Cdawg = Cdawg::new(Rc::new(RefCell::new(vec![c, o, c, o, a, o])));
        let (mut state, mut start) = (cdawg.source, 1);

        // Step 1: c
        (state, start) = cdawg.update(state, start, 1);
        let edge = cdawg
            .graph
            .get_edge(cdawg.graph.get_node(cdawg.source).get_first_edge());
        assert_eq!(edge.get_target().index(), cdawg.sink.index());
        assert_eq!(get_span!(cdawg, edge), (1, 1)); // Wrapper updates value of E
        assert_eq!(
            cdawg.extension(cdawg.source, to_inenaga((0, 1))).index(),
            cdawg.sink.index()
        );
        assert_eq!(
            cdawg
                .graph
                .get_node(cdawg.sink)
                .get_failure()
                .unwrap()
                .index(),
            cdawg.source.index()
        );
        assert_eq!(start, 2);

        // Step 2: co
        (state, start) = cdawg.update(state, start, 2);
        // Correctly has "co" edge instead of "c"?
        let co_idx = cdawg.get_edge_by_token(cdawg.source, c);
        let co_edge = cdawg.graph.get_edge(co_idx.unwrap());
        assert_eq!(get_span!(cdawg, co_edge), (1, 2));
        // Correctly has "o" edge?
        let o_idx = cdawg.get_edge_by_token(cdawg.source, o);
        let o_edge = cdawg.graph.get_edge(o_idx.unwrap());
        assert_eq!(get_span!(cdawg, o_edge), (2, 2));
        assert_eq!(start, 3);

        // Step 3: coc
        (state, start) = cdawg.update(state, start, 3);
        assert_eq!(get_span!(cdawg, co_edge), (1, 3));
        assert_eq!(get_span!(cdawg, o_edge), (2, 3));
        assert_eq!(start, 3); // (3, 3) represents "c"

        // Step 4: coco
        (state, start) = cdawg.update(state, start, 4);
        assert_eq!(get_span!(cdawg, co_edge), (1, 4));
        assert_eq!(get_span!(cdawg, o_edge), (2, 4));
        assert_eq!(start, 3); // (3, 4) represents "co"

        // Step 5: cocoa
        (state, start) = cdawg.update(state, start, 5);
        // Verify three edges out of source have the right labels and targets.
        let edge_co = get_edge!(cdawg, cdawg.source, c);
        let edge_o = get_edge!(cdawg, cdawg.source, o);
        let edge_a = get_edge!(cdawg, cdawg.source, a);
        assert_eq!(get_span!(cdawg, edge_co), (1, 2));
        assert_eq!(get_span!(cdawg, edge_o), (2, 2));
        assert_eq!(get_span!(cdawg, edge_a), (5, 5));
        assert_eq!(edge_a.get_target(), cdawg.sink);
        assert_eq!(edge_co.get_target(), edge_o.get_target());
        // Verify new state is set up correctly.
        let q = edge_co.get_target();
        assert!(q != cdawg.source);
        assert!(q != cdawg.sink);
        assert_eq!(cdawg.graph.get_node(q).get_failure(), Some(cdawg.source));
        // Verify two edges out of q.
        let edge_coa = get_edge!(cdawg, q, c);
        let edge_a2 = get_edge!(cdawg, q, a);
        assert_eq!(get_span!(cdawg, edge_coa), (3, 5));
        assert_eq!(get_span!(cdawg, edge_a), (5, 5));

        // Step 6: cocoao
        (state, start) = cdawg.update(state, start, 6);
        // Verify three edges out of source have the right labels and targets.
        let edge_co = get_edge!(cdawg, cdawg.source, c);
        let edge_o = get_edge!(cdawg, cdawg.source, o);
        let edge_ao = get_edge!(cdawg, cdawg.source, a);
        assert_eq!(get_span!(cdawg, edge_co), (1, 2));
        assert_eq!(get_span!(cdawg, edge_o), (6, 6));
        assert_eq!(get_span!(cdawg, edge_ao), (5, 6));
        assert_eq!(edge_a.get_target(), cdawg.sink);
        assert!(edge_co.get_target() != cdawg.sink);
        assert!(edge_o.get_target() != cdawg.sink);
        assert!(edge_co.get_target() != edge_o.get_target());
        // Verify new state is set up correctly.
        let q_co = edge_co.get_target();
        let q_o = edge_o.get_target();
        assert_eq!(cdawg.graph.get_node(q_co).get_failure(), Some(q_o));
        assert_eq!(cdawg.graph.get_node(q_o).get_failure(), Some(cdawg.source));
        // Check edges out of q_co and q_o.
        let edge_co_ao = get_edge!(cdawg, q_co, a);
        let edge_co_coao = get_edge!(cdawg, q_co, c);
        assert_eq!(get_span!(cdawg, edge_co_ao), (5, 6));
        assert_eq!(get_span!(cdawg, edge_co_coao), (3, 6));
        assert_eq!(edge_co_ao.get_target(), cdawg.sink);
        assert_eq!(edge_co_coao.get_target(), cdawg.sink);
        let edge_o_ao = get_edge!(cdawg, q_o, a);
        let edge_o_coao = get_edge!(cdawg, q_o, c);
        assert_eq!(get_span!(cdawg, edge_o_ao), (5, 6));
        assert_eq!(get_span!(cdawg, edge_o_coao), (3, 6));
        assert_eq!(edge_o_ao.get_target(), cdawg.sink);
        assert_eq!(edge_o_coao.get_target(), cdawg.sink);
    }

    #[test]
    fn test_update_aaa() {
        // Randomly thought this might reveal an issue where there were too many edges.
        let mut cdawg: Cdawg = Cdawg::new(Rc::new(RefCell::new(vec![0, 0, 0])));
        let (mut state, mut start) = (cdawg.source, 1);

        (state, start) = cdawg.update(state, start, 1);
        assert_eq!(get_edge!(cdawg, cdawg.source, 0).get_target(), cdawg.sink);
        assert_eq!(get_span!(cdawg, get_edge!(cdawg, cdawg.source, 0)), (1, 1));
        (state, start) = cdawg.update(state, start, 2);
        assert_eq!(get_edge!(cdawg, cdawg.source, 0).get_target(), cdawg.sink);
        assert_eq!(get_span!(cdawg, get_edge!(cdawg, cdawg.source, 0)), (1, 2));
        (state, start) = cdawg.update(state, start, 3);
        assert_eq!(get_edge!(cdawg, cdawg.source, 0).get_target(), cdawg.sink);
        assert_eq!(get_span!(cdawg, get_edge!(cdawg, cdawg.source, 0)), (1, 3));

        assert_eq!(cdawg.graph.n_edges(cdawg.source), 1);
    }

    #[test]
    fn test_update_abcabcaba() {
        // Taken from Figure 13 of Inenaga et al.
        let (a, b, c) = (0, 1, 2);
        let mut cdawg: Cdawg = Cdawg::new(Rc::new(RefCell::new(vec![a, b, c, a, b, c, a, b, a])));
        let (mut state, mut start) = (cdawg.source, 1);

        // 1) a
        (state, start) = cdawg.update(state, start, 1);
        assert_eq!(get_edge!(cdawg, cdawg.source, a).get_target(), cdawg.sink);
        assert_eq!(get_span!(cdawg, get_edge!(cdawg, cdawg.source, a)), (1, 1));
        assert_eq!(cdawg.graph.n_edges(cdawg.source), 1);

        // 2) ab
        (state, start) = cdawg.update(state, start, 2);
        assert_eq!(get_edge!(cdawg, cdawg.source, a).get_target(), cdawg.sink);
        assert_eq!(get_span!(cdawg, get_edge!(cdawg, cdawg.source, a)), (1, 2));
        assert_eq!(get_edge!(cdawg, cdawg.source, b).get_target(), cdawg.sink);
        assert_eq!(get_span!(cdawg, get_edge!(cdawg, cdawg.source, b)), (2, 2));
        assert_eq!(cdawg.graph.n_edges(cdawg.source), 2);

        // 3) abc
        (state, start) = cdawg.update(state, start, 3);
        assert_eq!(get_edge!(cdawg, cdawg.source, a).get_target(), cdawg.sink);
        assert_eq!(get_span!(cdawg, get_edge!(cdawg, cdawg.source, a)), (1, 3));
        assert_eq!(get_edge!(cdawg, cdawg.source, b).get_target(), cdawg.sink);
        assert_eq!(get_span!(cdawg, get_edge!(cdawg, cdawg.source, b)), (2, 3));
        assert_eq!(get_edge!(cdawg, cdawg.source, c).get_target(), cdawg.sink);
        assert_eq!(get_span!(cdawg, get_edge!(cdawg, cdawg.source, c)), (3, 3));
        assert_eq!(cdawg.graph.n_edges(cdawg.source), 3);

        // 4) abca: does not need to split a node
        (state, start) = cdawg.update(state, start, 4);
        assert_eq!(get_edge!(cdawg, cdawg.source, a).get_target(), cdawg.sink);
        assert_eq!(get_span!(cdawg, get_edge!(cdawg, cdawg.source, a)), (1, 4));
        assert_eq!(get_edge!(cdawg, cdawg.source, b).get_target(), cdawg.sink);
        assert_eq!(get_span!(cdawg, get_edge!(cdawg, cdawg.source, b)), (2, 4));
        assert_eq!(get_edge!(cdawg, cdawg.source, c).get_target(), cdawg.sink);
        assert_eq!(get_span!(cdawg, get_edge!(cdawg, cdawg.source, c)), (3, 4));
        assert_eq!(cdawg.graph.n_edges(cdawg.source), 3);

        // 5) abcab: does not need to split a node
        (state, start) = cdawg.update(state, start, 5);
        assert_eq!(get_edge!(cdawg, cdawg.source, a).get_target(), cdawg.sink);
        assert_eq!(get_span!(cdawg, get_edge!(cdawg, cdawg.source, a)), (1, 5));
        assert_eq!(get_edge!(cdawg, cdawg.source, b).get_target(), cdawg.sink);
        assert_eq!(get_span!(cdawg, get_edge!(cdawg, cdawg.source, b)), (2, 5));
        assert_eq!(get_edge!(cdawg, cdawg.source, c).get_target(), cdawg.sink);
        assert_eq!(get_span!(cdawg, get_edge!(cdawg, cdawg.source, c)), (3, 5));
        assert_eq!(cdawg.graph.n_edges(cdawg.source), 3);

        // 6) abcabc: does not need to split a node
        (state, start) = cdawg.update(state, start, 6);
        assert_eq!(get_edge!(cdawg, cdawg.source, a).get_target(), cdawg.sink);
        assert_eq!(get_span!(cdawg, get_edge!(cdawg, cdawg.source, a)), (1, 6));
        assert_eq!(get_edge!(cdawg, cdawg.source, b).get_target(), cdawg.sink);
        assert_eq!(get_span!(cdawg, get_edge!(cdawg, cdawg.source, b)), (2, 6));
        assert_eq!(get_edge!(cdawg, cdawg.source, c).get_target(), cdawg.sink);
        assert_eq!(get_span!(cdawg, get_edge!(cdawg, cdawg.source, c)), (3, 6));
        assert_eq!(cdawg.graph.n_edges(cdawg.source), 3);

        // 7) abcabca: does not need to split a node
        (state, start) = cdawg.update(state, start, 7);
        assert_eq!(get_edge!(cdawg, cdawg.source, a).get_target(), cdawg.sink);
        assert_eq!(get_span!(cdawg, get_edge!(cdawg, cdawg.source, a)), (1, 7));
        assert_eq!(get_edge!(cdawg, cdawg.source, b).get_target(), cdawg.sink);
        assert_eq!(get_span!(cdawg, get_edge!(cdawg, cdawg.source, b)), (2, 7));
        assert_eq!(get_edge!(cdawg, cdawg.source, c).get_target(), cdawg.sink);
        assert_eq!(get_span!(cdawg, get_edge!(cdawg, cdawg.source, c)), (3, 7));
        assert_eq!(cdawg.graph.n_edges(cdawg.source), 3);

        // 8) abcabcab: does not need to split a node
        (state, start) = cdawg.update(state, start, 8);
        assert_eq!(get_edge!(cdawg, cdawg.source, a).get_target(), cdawg.sink);
        assert_eq!(get_span!(cdawg, get_edge!(cdawg, cdawg.source, a)), (1, 8));
        assert_eq!(get_edge!(cdawg, cdawg.source, b).get_target(), cdawg.sink);
        assert_eq!(get_span!(cdawg, get_edge!(cdawg, cdawg.source, b)), (2, 8));
        assert_eq!(get_edge!(cdawg, cdawg.source, c).get_target(), cdawg.sink);
        assert_eq!(get_span!(cdawg, get_edge!(cdawg, cdawg.source, c)), (3, 8));
        assert_eq!(cdawg.graph.n_edges(cdawg.source), 3);
        assert_eq!(start, 4);

        // abcabcaba: where the magic happens! Shown in Figure 13
        (state, start) = cdawg.update(state, start, 9);
        assert_eq!(cdawg.graph.n_edges(cdawg.source), 3);
        assert_eq!(get_span!(cdawg, get_edge!(cdawg, cdawg.source, a)), (1, 2));
        assert_eq!(get_span!(cdawg, get_edge!(cdawg, cdawg.source, b)), (2, 2));
        assert_eq!(get_span!(cdawg, get_edge!(cdawg, cdawg.source, c)), (3, 5));
        let qa = get_edge!(cdawg, cdawg.source, a).get_target();
        let qb = get_edge!(cdawg, cdawg.source, b).get_target();
        let qc = get_edge!(cdawg, cdawg.source, c).get_target();
        assert_eq!(qa, qb);
        assert!(qa != cdawg.source);
        assert!(qc != cdawg.source);
        assert!(qa != qc);
        let q1 = qa;
        let q2 = qc;
        // Check q1 edges.
        assert_eq!(cdawg.graph.n_edges(q1), 2);
        assert_eq!(get_span!(cdawg, get_edge!(cdawg, q1, a)), (9, 9));
        assert_eq!(get_span!(cdawg, get_edge!(cdawg, q1, c)), (3, 5));
        assert_eq!(get_edge!(cdawg, q1, a).get_target(), cdawg.sink);
        assert_eq!(get_edge!(cdawg, q1, c).get_target(), q2);
        // Check q2 edges.
        assert_eq!(cdawg.graph.n_edges(q2), 2);
        assert_eq!(get_span!(cdawg, get_edge!(cdawg, q1, a)), (9, 9));
        assert_eq!(get_span!(cdawg, get_edge!(cdawg, q1, c)), (3, 5));
        assert_eq!(get_edge!(cdawg, q2, a).get_target(), cdawg.sink);
        assert_eq!(get_edge!(cdawg, q2, c).get_target(), cdawg.sink);
    }

    #[test]
    fn test_sink_length() {
        let (a, b, c) = (0, 1, 2);
        let mut cdawg: Cdawg = Cdawg::new(Rc::new(RefCell::new(vec![a, b, c, a, b, c, a, b, a])));
        let (mut state, mut start) = (cdawg.source, 1);

        (state, start) = cdawg.update(state, start, 1);
        assert_eq!(cdawg.graph.get_node(cdawg.sink).get_length(), 1);
        (state, start) = cdawg.update(state, start, 2);
        assert_eq!(cdawg.graph.get_node(cdawg.sink).get_length(), 2);
    }

    type DiskW = DefaultWeight;
    type DiskE = (DefaultIx, DefaultIx);
    type DiskCdawg = Cdawg<DiskW, DefaultIx, DiskBacking<DiskW, DiskE, DefaultIx>>;

    #[test]
    fn test_save_metadata_load_null() {
        let tmp_dir = tempdir().unwrap();
        let path = tmp_dir.path();

        let tokens: Vec<u16> = vec![10, 11, 12];
        let mb = DiskBacking::new(path);
        let mut cdawg: DiskCdawg = Cdawg::new_mb(Rc::new(RefCell::new(tokens)), mb);
        cdawg.add_balanced_edge(cdawg.source, cdawg.sink, (1, 1));
        cdawg.save_metadata(path).unwrap();

        let tokens2: Vec<u16> = vec![10, 11, 12];
        let cdawg2: DiskCdawg =
            Cdawg::load(Rc::new(RefCell::new(tokens2)), path, CacheConfig::none()).unwrap();
        assert_eq!(cdawg2.source, cdawg.source);
        assert_eq!(cdawg2.sink, cdawg.sink);
        assert_eq!(
            get_edge!(cdawg2, cdawg2.source, 10).get_target(),
            cdawg.sink
        );
    }

    #[test]
    fn test_tokens_disk_vec() {
        // Perform step 1 of cocoa on a DiskVec.
        let tmp_dir = tempdir().unwrap();

        let vec: Vec<u16> = vec![0, 1, 2];
        let disk_vec = DiskVec::<u16>::from_vec(&vec, tmp_dir.path().join("vec.bin")).unwrap();
        let mut cdawg: Cdawg = Cdawg::new(Rc::new(RefCell::new(disk_vec)));
        let (mut state, mut start) = (cdawg.source, 1);

        // Step 1: c
        (state, start) = cdawg.update(state, start, 1);
        let edge = cdawg
            .graph
            .get_edge(cdawg.graph.get_node(cdawg.source).get_first_edge());
        assert_eq!(edge.get_target().index(), cdawg.sink.index());
        assert_eq!(get_span!(cdawg, edge), (1, 1)); // Wrapper updates value of E
        assert_eq!(
            cdawg.extension(cdawg.source, to_inenaga((0, 1))).index(),
            cdawg.sink.index()
        );
        assert_eq!(
            cdawg
                .graph
                .get_node(cdawg.sink)
                .get_failure()
                .unwrap()
                .index(),
            cdawg.source.index()
        );
        assert_eq!(start, 2);
    }

    #[test]
    fn test_transition_and_count_abcbca() {
        let (a, b, c, d) = (0, 1, 2, 3);
        let train = Rc::new(RefCell::new(vec![a, b, c, b, c, a]));
        let mut cdawg: Cdawg = Cdawg::new(train);
        cdawg.build();

        let mut lengths = Vec::new();
        let mut cs = cdawg.get_initial();
        for token in [a, b, c, a, d].iter() {
            cs = cdawg.transition_and_count(cs, *token);
            lengths.push(cs.length);
        }
        assert_eq!(lengths, vec![1, 2, 3, 3, 0]);
    }

    #[test]
    fn test_transition_and_count_abcabcaba() {
        let (a, b, c) = (0, 1, 2);
        let train = Rc::new(RefCell::new(vec![a, b, c, a, b, c, a, b, a]));
        let mut cdawg: Cdawg = Cdawg::new(train);
        cdawg.build();

        let mut lengths = Vec::new();
        let mut cs = cdawg.get_initial();
        for token in [a, b, a].iter() {
            cs = cdawg.transition_and_count(cs, *token);
            lengths.push(cs.length);
        }
        assert_eq!(lengths, vec![1, 2, 3]);

        lengths = Vec::new();
        cs = cdawg.get_initial();
        for token in [a, b, b].iter() {
            cs = cdawg.transition_and_count(cs, *token);
            lengths.push(cs.length);
        }
        assert_eq!(lengths, vec![1, 2, 1]);
    }

    #[test]
    fn test_transition_and_count_abcbd() {
        // Should test the case where we implicitly fail from a state but canonize not required.
        let (a, b, c, d) = (0, 1, 2, 3);
        let train = Rc::new(RefCell::new(vec![a, b, c, b, d]));
        let mut cdawg: Cdawg = Cdawg::new(train);
        cdawg.build();

        let mut lengths = Vec::new();
        let mut cs = cdawg.get_initial();
        for token in [a, b, d].iter() {
            cs = cdawg.transition_and_count(cs, *token);
            lengths.push(cs.length);
        }
        assert_eq!(lengths, vec![1, 2, 2]);
    }

    #[test]
    fn test_multidoc_abc_bcd() {
        let (a, b, c, d) = (0, 1, 2, 3);
        let train = Rc::new(RefCell::new(vec![a, b, c, u16::MAX, b, c, d, u16::MAX]));
        let mut cdawg: Cdawg = Cdawg::new(train.clone());

        let (mut state, mut start) = (cdawg.source, 1);
        let length = train.borrow().len();
        let mut starts = Vec::new();
        for idx in 1..length + 1 {
            (state, start) = cdawg.update(state, start, idx);
            if train.borrow().get(idx - 1) == u16::MAX {
                (state, start) = cdawg.end_document(idx, idx);
                starts.push(start);
            }
        }

        // Make sure the start gets reset to 1-indexed + 1.
        assert_eq!(starts, vec![5, 9]);

        // Check that the documents are encoded correctly with edges from source to their sink node.
        let cmp0 = CdawgComparator::new(train.clone());
        let doc0 = cdawg.graph.get_edge_by_weight_cmp(
            cdawg.source,
            (DefaultIx::new(3), DefaultIx::new(0)),
            Box::new(cmp0),
        );
        assert_eq!(cdawg.graph.get_edge(doc0.unwrap()).get_target().index(), 1);
        let cmp1 = CdawgComparator::new(train.clone());
        let doc1 = cdawg.graph.get_edge_by_weight_cmp(
            cdawg.source,
            (DefaultIx::new(7), DefaultIx::new(0)),
            Box::new(cmp1),
        );
        assert_eq!(cdawg.graph.get_edge(doc1.unwrap()).get_target().index(), 2);

        // Check that the suffix overlaps are returned correctly.
        let mut lengths = Vec::new();
        let mut cs = cdawg.get_initial();
        for token in [b, c, b].iter() {
            cs = cdawg.transition_and_count(cs, *token);
            lengths.push(cs.length);
        }
        assert_eq!(lengths, vec![1, 2, 1]);
    }

    #[test]
    fn test_multidoc_cocoa_cola() {
        // Taken from Figure 19 in the paper.
        let end = u16::MAX;
        let (c, o, a, l) = (0, 1, 2, 3);
        let train = Rc::new(RefCell::new(vec![c, o, c, o, a, end, c, o, l, a]));
        let mut cdawg: Cdawg = Cdawg::new(train.clone());

        let (mut state, mut start) = (cdawg.source, 1);
        let length = train.borrow().len();
        for idx in 1..length + 1 {
            (state, start) = cdawg.update(state, start, idx);
            if train.borrow().get(idx - 1) == u16::MAX {
                (state, start) = cdawg.end_document(idx, idx);
            }
        }

        let mut lengths = Vec::new();
        let mut counts = Vec::new();
        let mut cs = cdawg.get_initial();
        for token in [c, o, a, c, o, l].iter() {
            cs = cdawg.transition_and_count(cs, *token);
            lengths.push(cs.length);
            counts.push(cdawg.get_suffix_count(cs));
        }
        assert_eq!(lengths, vec![1, 2, 3, 1, 2, 3]);
        assert_eq!(counts, vec![0, 0, 1, 0, 0, 1]); // 1 means sink
    }

    #[test]
    fn test_build_a_end_b_end() {
        let train = Rc::new(RefCell::new(vec![0, u16::MAX, 1, u16::MAX]));
        let mut cdawg: Cdawg = Cdawg::new(train.clone());
        cdawg.build();

        assert_eq!(cdawg.node_count(), 4); // 3 real nodes + new sink

        // Test the normal edges.
        let edge_a = get_edge!(cdawg, cdawg.source, 0);
        assert_eq!(edge_a.get_target().index(), 1);
        assert_eq!(
            cdawg.get_span(edge_a.get_weight(), edge_a.get_target()),
            (1, 2)
        ); // 1-indexed
        let edge_b = get_edge!(cdawg, cdawg.source, 1);
        assert_eq!(edge_b.get_target().index(), 2);
        assert_eq!(
            cdawg.get_span(edge_b.get_weight(), edge_b.get_target()),
            (3, 4)
        ); // 1-indexed

        // Test the sink edges.
        let cmp0 = CdawgComparator::new(train.clone());
        let doc0 = cdawg.graph.get_edge_by_weight_cmp(
            cdawg.source,
            (DefaultIx::new(1), DefaultIx::new(2)),
            Box::new(cmp0),
        );
        assert_eq!(
            cdawg.graph.get_edge(doc0.unwrap()).get_target(),
            NodeIndex::new(1)
        );
        let cmp1 = CdawgComparator::new(train.clone());
        let doc1 = cdawg.graph.get_edge_by_weight_cmp(
            cdawg.source,
            (DefaultIx::new(3), DefaultIx::new(4)),
            Box::new(cmp1),
        );
        assert_eq!(
            cdawg.graph.get_edge(doc1.unwrap()).get_target(),
            NodeIndex::new(2)
        );

        // Counts just reflect whether a state is sink at this point.
        assert_eq!(cdawg.get_count(NodeIndex::new(0)), 0);
        assert_eq!(cdawg.get_count(NodeIndex::new(1)), 1);
        assert_eq!(cdawg.get_count(NodeIndex::new(2)), 1);
    }

    #[test]
    fn test_get_count_cocoa() {
        // Test counts incrementally.
        let (c, o, a) = (0, 1, 2);
        let train = Rc::new(RefCell::new(vec![c, o, c, o, a, u16::MAX]));
        let mut cdawg: Cdawg = Cdawg::new(train);

        let q0 = NodeIndex::new(0);
        let q1 = NodeIndex::new(1);
        let q2 = NodeIndex::new(2);

        // Step 0: check counts in empty CDAWG'
        let (mut state, mut start) = (cdawg.source, 1);
        assert_eq!(cdawg.get_count(q0), 0);
        assert_eq!(cdawg.get_count(q1), 1);

        // Step 1: compare counts against "c"
        (state, start) = cdawg.update(state, start, 1);
        assert_eq!(cdawg.get_count(q0), 0);
        assert_eq!(cdawg.get_count(q1), 1);

        // Step 2: compare counts against "co"
        (state, start) = cdawg.update(state, start, 2);
        assert_eq!(cdawg.get_count(q0), 0);
        assert_eq!(cdawg.get_count(q1), 1);

        // Step 3: compare counts against "coc"
        (state, start) = cdawg.update(state, start, 3);
        assert_eq!(cdawg.get_count(q0), 0);
        assert_eq!(cdawg.get_count(q1), 1);

        // Step 4: compare counts against "coco"
        (state, start) = cdawg.update(state, start, 4);
        assert_eq!(cdawg.get_count(q0), 0);
        assert_eq!(cdawg.get_count(q1), 1);

        // Step 5: compare counts against "cocoa". Where the magic happens!!
        (state, start) = cdawg.update(state, start, 5);
        assert_eq!(cdawg.get_count(q0), 0);
        assert_eq!(cdawg.get_count(q1), 1);
        assert_eq!(cdawg.get_count(q2), 0);

        // Step 6: compare counts against "cocoa$".
        (state, start) = cdawg.update(state, start, 6);
        assert_eq!(cdawg.get_count(q0), 0);
        assert_eq!(cdawg.get_count(q1), 1);
        assert_eq!(cdawg.get_count(q2), 0);
    }

    #[test]
    fn test_get_count_abcabcaba() {
        // Test counts incrementally.
        let (a, b, c) = (0, 1, 2);
        let train = Rc::new(RefCell::new(vec![a, b, c, a, b, c, a, b, a, u16::MAX]));
        let mut cdawg: Cdawg = Cdawg::new(train);

        let q0 = NodeIndex::new(0);
        let q1 = NodeIndex::new(1);
        let q2 = NodeIndex::new(2);
        let q3 = NodeIndex::new(3);
        let q4 = NodeIndex::new(4);

        // abcabcab
        let (mut state, mut start) = (cdawg.source, 1);
        for idx in 1..9 {
            (state, start) = cdawg.update(state, start, idx);
        }
        assert_eq!(cdawg.get_count(q0), 0);
        assert_eq!(cdawg.get_count(q1), 1);

        // Adding "a" triggers a complex series of updates.
        (state, start) = cdawg.update(state, start, 9);
        assert_eq!(cdawg.get_count(q0), 0);
        assert_eq!(cdawg.get_count(q1), 1);
        assert_eq!(cdawg.get_count(q2), 0);
        assert_eq!(cdawg.get_count(q3), 0);

        // Add a "$"
        (state, start) = cdawg.update(state, start, 10);
        assert_eq!(cdawg.get_count(q0), 0);
        assert_eq!(cdawg.get_count(q1), 1);
        assert_eq!(cdawg.get_count(q2), 0);
        assert_eq!(cdawg.get_count(q3), 0);
        assert_eq!(cdawg.get_count(q4), 0);
    }

    #[test]
    fn test_get_entropy() {
        // Test counts incrementally.
        let (a, b, c, d) = (0, 1, 2, 3);
        let train = Rc::new(RefCell::new(vec![c, a, b, a, c, u16::MAX]));
        let mut cdawg: Cdawg = Cdawg::new(train);
        cdawg.build();
        let mut counter = TopologicalCounter::new_ram();
        counter.fill_counts(&mut cdawg);

        let mut entropies = Vec::new();
        let mut cs = cdawg.get_initial();
        for token in [a, b, a, d, c].iter() {
            cs = cdawg.transition_and_count(cs, *token);
            entropies.push(cdawg.get_entropy(cs));
        }
        // The 3rd value is 2 * 1/6 * log2(1/6) + 2 * 2/6 * log2(2/6)
        assert_eq!(entropies, vec![1., 0., 0., 1.9182958340544896, 1.]);
    }

    #[test]
    fn test_get_next_tokens() {
        // Test counts incrementally.
        let (a, b, c, d) = (0, 1, 2, 3);
        let train = Rc::new(RefCell::new(vec![c, a, b, a, c, u16::MAX]));
        let mut cdawg: Cdawg = Cdawg::new(train);
        cdawg.build();
        let mut counter = TopologicalCounter::new_ram();
        counter.fill_counts(&mut cdawg);

        let mut next_tokens = Vec::new();
        let mut cs = cdawg.get_initial();
        for token in [a, b, a, d, c].iter() {
            cs = cdawg.transition_and_count(cs, *token);
            let mut tokens = cdawg.get_next_tokens(cs);
            tokens.sort_by(|tup1, tup2| tup1.0.cmp(&tup2.0));
            next_tokens.push(tokens);
        }

        assert_eq!(
            next_tokens,
            vec![
                vec![(b, 0.5), (c, 0.5)],
                vec![(a, 1.0)],
                vec![(c, 1.0)],
                vec![
                    (a, 2. / 6.),
                    (b, 1. / 6.),
                    (c, 2. / 6.),
                    (u16::MAX, 1. / 6.)
                ],
                vec![(a, 0.5), (u16::MAX, 0.5)],
            ]
        );
    }
}
