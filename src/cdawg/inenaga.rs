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

use anyhow::Result;
use std::convert::TryInto;
use std::path::Path;
use serde::{Deserialize, Serialize};
use std::rc::Rc;
use std::cell::RefCell;

use graph::{EdgeRef, NodeRef};
use graph::avl_graph::edge::EdgeMutRef;
use graph::avl_graph::node::NodeMutRef;
use graph::avl_graph::AvlGraph;
use graph::indexing::{DefaultIx, NodeIndex, EdgeIndex, IndexType};
use weight::{DefaultWeight, Weight};
use graph::memory_backing::{DiskBacking, MemoryBacking, RamBacking};
use cdawg::cdawg_edge_weight::CdawgEdgeWeight;
use cdawg::metadata::CdawgMetadata;
use cdawg::token_backing::TokenBacking;
use cdawg::comparator::CdawgComparator;
use cdawg::cdawg_state::CdawgState;

// TODO: Add TokenBacking for tokens

pub struct Cdawg<W = DefaultWeight, Ix = DefaultIx, Mb = RamBacking<W, CdawgEdgeWeight<Ix>, Ix>>
where
    Ix: IndexType,
    W: Weight + Clone,
    Mb: MemoryBacking<W, CdawgEdgeWeight<Ix>, Ix>,
{
    tokens: Rc<RefCell<dyn TokenBacking<u16>>>,
    graph: AvlGraph<W, CdawgEdgeWeight<Ix>, Ix, Mb>,
    source: NodeIndex<Ix>,
    sink: NodeIndex<Ix>,
    e: usize,
}

impl<W, Ix> Cdawg<W, Ix>
where
    Ix: IndexType,
    W: Weight + Serialize + for<'de> Deserialize<'de> + Clone,
{
    pub fn new(tokens: Rc<RefCell<dyn TokenBacking<u16>>>) -> Self {
        let mb: RamBacking<W, CdawgEdgeWeight<Ix>, Ix> = RamBacking::default();
        Self::new_mb(tokens, mb)
    }
}

impl<W, Ix> Cdawg<W, Ix, DiskBacking<W, CdawgEdgeWeight<Ix>, Ix>>
where
    Ix: IndexType + Serialize + for<'de> serde::Deserialize<'de>,
    W: Weight + Serialize + for<'de> Deserialize<'de> + Clone + Default,
    CdawgEdgeWeight<Ix>: Serialize + for<'de> Deserialize<'de>,
{
    pub fn load<P: AsRef<Path> + Clone + std::fmt::Debug>(tokens: Rc<RefCell<dyn TokenBacking<u16>>>, path: P) -> Result<Self> {
        let path2 = path.clone();
        let mut config_path = path2.as_ref().to_path_buf();
        config_path.push("metadata.json");
        let config = CdawgMetadata::load_json(config_path)?;

        let graph = AvlGraph::load(path)?;

        Ok(Self {
            tokens,
            graph,
            source: NodeIndex::new(config.source),
            sink: NodeIndex::new(config.sink),
            e: config.n_nodes,
        })
    }

    pub fn save<P: AsRef<Path> + Clone>(&self, path: P) -> Result<()> {
        let mut config_path = path.as_ref().to_path_buf();
        config_path.push("metadata.json");
        let config = CdawgMetadata {
            source: self.source.index(),
            sink: self.sink.index(),
            n_nodes: self.e,
        };
        config.save_json(config_path)
    }
}

impl<W, Ix, Mb> Cdawg<W, Ix, Mb>
where
    Ix: IndexType,
    W: Weight + Serialize + for<'de> Deserialize<'de> + Clone,
    Mb: MemoryBacking<W, CdawgEdgeWeight<Ix>, Ix>,
    Mb::EdgeRef: Copy,
{

    pub fn new_mb(tokens: Rc<RefCell<dyn TokenBacking<u16>>>, mb: Mb) -> Cdawg<W, Ix, Mb> {
        let mut graph: AvlGraph<W, CdawgEdgeWeight<Ix>, Ix, Mb> = AvlGraph::new_mb(mb);
        let source = graph.add_node(W::new(0, None, 0));
        let source_ = NodeIndex::new(source.index());  // FIXME: Weight type linked to Ix
        let length = Ix::max_value().index();  // Needs to adapt.
        let sink = graph.add_node(W::new(length.try_into().unwrap(), Some(source_), 0));
        Self {tokens, graph, source, sink, e: 0}
    }

    pub fn with_capacity_mb(tokens: Rc<RefCell<dyn TokenBacking<u16>>>,
                            mb: Mb,
                            n_nodes: usize,
                            n_edges: usize,) -> Cdawg<W, Ix, Mb> {
        let mut graph: AvlGraph<W, CdawgEdgeWeight<Ix>, Ix, Mb> = AvlGraph::with_capacity_mb(mb, n_nodes, n_edges);
        let source = graph.add_node(W::new(0, None, 0));
        let source_ = NodeIndex::new(source.index());  // FIXME: Weight type linked to Ix
        let length = Ix::max_value().index();
        let sink = graph.add_node(W::new(length.try_into().unwrap(), Some(source_), 0));
        Self {tokens, graph, source, sink, e: 0}
    }

    // Tokens needs to be fully populated for this to work!
    pub fn build(&mut self) {
        let (mut state, mut start) = (self.source, 1);
        let length = self.tokens.borrow().len();
        for idx in 1..length + 1 {
            (state, start) = self.update(state, start, idx);
        }
    }

    pub fn update(&mut self,
              in_state: NodeIndex<Ix>,  // Cannot be null.
              mut start: usize,
              end: usize,) -> (NodeIndex<Ix>, usize) {
        self.e = end;
        let token = self.tokens.borrow().get(end - 1);  // Map p back to 0-indexing
        let mut dest: Option<NodeIndex<Ix>> = None;
        let mut r = NodeIndex::end();
        let mut opt_state: Option<NodeIndex<Ix>> = Some(in_state);
        let mut opt_r: Option<NodeIndex<Ix>> = None;
        let mut opt_old_r: Option<NodeIndex<Ix>> = None;
        while !self.check_end_point(opt_state, (start, end - 1), token) {
            // Within the loop, never possible for opt_state to be null.
            let state = opt_state.unwrap();
            if start <= end - 1 {
                // Implicit case checks when an edge is active.
                let cur_dest = self.extension(state, (start, end - 1));
                if dest == Some(cur_dest) {
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

            // 1) Create new edge from r to sink with (end, *self.e)
            self.add_balanced_edge(r, self.sink, (end, Ix::max_value().index()));
            
            // 2) Set failure transition.
            if let Some(old_r) = opt_old_r {
                self.graph.get_node_mut(old_r).set_failure(Some(r));
            }
            opt_old_r = Some(r);

            // 3) Update state by canonizing the fstate.
            let old_start = start;  // TODO: remove after debug
            let fstate = self.graph.get_node(state).get_failure();
            (opt_state, start) = self.canonize(fstate, (start, end - 1));
        }

        if let Some(old_r) = opt_old_r {
            self.graph.get_node_mut(old_r).set_failure(opt_state);
        }
        self.separate_node(opt_state, (start, end))
    }

    // This is just following a transition (doesn't eat up everything potentially)
    // Note: 1-indexed!
    fn extension(&self, state: NodeIndex<Ix>, gamma: (usize, usize)) -> NodeIndex<Ix> {
        let (start, end) = gamma;
        if start > end {
            return state;
        }
        let (_, _, target) = self._get_start_end_target(state, self.tokens.borrow().get(start - 1));
        target
    }

    // Change the target of the edge coming out of state with path gamma.
    // Note: 1-indexed!
    pub fn redirect_edge(&mut self, state: NodeIndex<Ix>, gamma: (usize, usize), target: NodeIndex<Ix>) {
        let (start, end) = gamma;
        let token = self.tokens.borrow().get(start - 1);
        let edge_idx = self.get_edge_by_token(state, token).unwrap();
        let edge_ref = self.graph.get_edge(edge_idx);
        let (found_start, _) = self._get_span(edge_ref.get_weight());

        // Doesn't actually use graph.reroute_edge
        let weight = self._new_edge_weight(found_start, found_start + end - start);
        self.graph.get_edge_mut(edge_idx).set_weight(weight);
        self.graph.get_edge_mut(edge_idx).set_target(target);
    }

    // Split the edge and leave failure transitions unedited.
    fn split_edge(&mut self, q: NodeIndex<Ix>, gamma: (usize, usize)) -> NodeIndex<Ix> {
        // First, create a new node and set it's length.
        let v = self.graph.add_node(self.graph.get_node(q).get_weight());
        let q_length = self._get_length(q);
        let gamma_length = <usize as TryInto<u64>>::try_into(gamma.1 - gamma.0 + 1).unwrap();
        self.graph.get_node_mut(v).set_length(q_length + gamma_length);

        // Next, get the existing edge we're going to split.
        let token = self.tokens.borrow().get(gamma.0 - 1); // 0-indexed
        let edge_idx = self.get_edge_by_token(q, token);
        let edge = self.graph.get_edge(edge_idx.unwrap());
        let (mut start, end) = edge.get_weight().get_span();  // We DON'T call get_span because we want to keep end pointers.
        start += 1;  // Map back to Inenaga 1-indexed!
        let target = edge.get_target();

        // Reroute this edge into v.
        self.graph.get_edge_mut(edge_idx.unwrap()).set_weight(self._new_edge_weight(start, start + gamma.1 - gamma.0));
        self.graph.get_edge_mut(edge_idx.unwrap()).set_target(v);

        // Create a new edge from v to the original target.
        self.add_balanced_edge(v, target, (start + gamma.1 - gamma.0 + 1, end));

        v
    }

    fn separate_node(&mut self, mut state: Option<NodeIndex<Ix>>, gamma: (usize, usize)) -> (NodeIndex<Ix>, usize) {
        let (mut start, end) = gamma;
        let (opt_state1, start1) = self.canonize(state, (start, end));
        let state1 = opt_state1.unwrap();

        // Implicit case: active point is along an edge.
        if start1 <= end {
            return (state1, start1);
        }

        let length = match state {
            Some(q) => self._get_length(q) as i64,
            None => -1,
        };
        let length1 = self._get_length(state1) as i64;
        if length1 == length + (end - start + 1) as i64 {
            return (state1, start1);
        }

        // Non-solid, explicit case: clone node and set its length
        let mut weight = self.graph.get_node(state1).get_weight().clone();
        weight.set_length((length + (end - start + 1) as i64) as u64);
        let new_state = self.graph.add_node(weight);
        self.graph.clone_edges(state1, new_state);

        // Update the failure transitions.
        self.graph.get_node_mut(new_state).set_failure(self.graph.get_node(state1).get_failure());
        self.graph.get_node_mut(state1).set_failure(Some(new_state));

        // Replace edges from state to state1 with edges to new_state.
        // We know that state is non-null here.
        loop {
            // Reroute tokens[start-1] edge to new_state via (start, end)
            self._set_start_end_target(state.unwrap(), start, end, new_state);
            
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
    fn canonize(&self, mut state: Option<NodeIndex<Ix>>, gamma: (usize, usize)) -> (Option<NodeIndex<Ix>>, usize) {
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
                let token = self.tokens.borrow().get(start - 1);
                (found_start, found_end, found_state) = self._get_start_end_target(q, token);
            },
            None => {
                (found_start, found_end, found_state) = (0, 0, self.source);
            }
        }
        
        while found_end + start <= end + found_start {  // Written this way to avoid overflow.
            start += found_end + 1 - found_start;  // Written this way to avoid overflow.
            state = Some(found_state);
            if start <= end {
                let token = self.tokens.borrow().get(start - 1);
                (found_start, found_end, found_state) = self._get_start_end_target(found_state, token);
            }
        }
        (state, start)
    }

    // Return true if we can keep going; false otherwise.
    // 1-indexed!
    fn check_end_point(&self, state: Option<NodeIndex<Ix>>, gamma: (usize, usize), token: u16) -> bool {
        let (start, end) = gamma;
        if start <= end {
            let wk = self.tokens.borrow().get(start - 1);
            let e = self.get_edge_by_token(state.unwrap(), wk).unwrap();
            let (found_start, _) = self._get_span(self.graph.get_edge(e).get_weight());
            token == self.tokens.borrow().get(found_start + end - start)  // No +1 because 0-indexed.
        } else {
            match state {
                Some(phi) => {
                    let edge_idx = self.get_edge_by_token(phi, token);
                    edge_idx.is_some()
                },
                None => true,
            }
        }
    }

    // These helper methods are useful.
    // Could possible deprecate or merge these together.

    // Add a new edge with appropriate weight.
    // Take in indices with paper's conventions and return ours.
    // Maybe make this a macro?
    fn _new_edge_weight(&mut self, start: usize, end: usize) -> CdawgEdgeWeight<Ix> {
        CdawgEdgeWeight::new(
            start - 1,  // Map start to 0-indexed
            end,  // Keep end as 1-indexed
        )
    }

    // Get the Inenaga-indexed span associated with an edge.
    // Maybe make this a macro?
    fn _get_span(&self, weight: CdawgEdgeWeight<Ix>) -> (usize, usize) {
        let (start, end) = weight.get_span();
        // Shift to 1-indexed and retrieve value of end pointer.
        (start + 1, usize::min(end, self.e))
    }

    fn _get_start_end_target(&self, state: NodeIndex<Ix>, token: u16) -> (usize, usize, NodeIndex<Ix>) {
        let edge_idx = self.get_edge_by_token(state, token);
        let edge_ref = self.graph.get_edge(edge_idx.unwrap());
        let (start, end) = edge_ref.get_weight().get_span();
        let target = edge_ref.get_target();
        // Shift to 1-indexed and retrieve value of end pointer.
        (start + 1, usize::min(end, self.e), target)
    }

    fn _set_start_end_target(&mut self, state: NodeIndex<Ix>, start: usize, end: usize, target: NodeIndex<Ix>) {
        let token = self.tokens.borrow().get(start - 1);  // Potential double retrieval of token here.
        let edge_idx = self.get_edge_by_token(state, token);
        // For some reason, cannot call both on some EdgeMutRef.
        // TODO: Add a setter for weight and target together on EdgeMutRef.
        self.graph.get_edge_mut(edge_idx.unwrap()).set_weight(self._new_edge_weight(start, end));
        self.graph.get_edge_mut(edge_idx.unwrap()).set_target(target);
    }

    fn _get_length(&self, q: NodeIndex<Ix>) -> u64 {
        let length = self.graph.get_node(q).get_length();
        // Handle sink length correctly: paper says length(sink) = e
        u64::min(length, self.e.try_into().unwrap())
    }

    // Convenience methods.
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

    pub fn get_edge_by_token(&self, state: NodeIndex<Ix>, token: u16) -> Option<EdgeIndex<Ix>> {
        let weight = CdawgEdgeWeight::new(0, 0);  // Doesn't matter since comparator has token.
        let cmp = CdawgComparator::new_with_token(self.tokens.clone(), token);
        self.graph.get_edge_by_weight_cmp(state, weight, Box::new(cmp))
    }

    pub fn add_balanced_edge(&mut self, state: NodeIndex<Ix>, target: NodeIndex<Ix>, gamma: (usize, usize)) {
        let weight = self._new_edge_weight(gamma.0, gamma.1);
        let token = self.tokens.borrow().get(gamma.0 - 1);  // Map to 0-indexed
        let cmp = CdawgComparator::new_with_token(self.tokens.clone(), token);
        self.graph.add_balanced_edge_cmp(state, target, weight, Box::new(cmp))
    }

    // Methods for inference with the CDAWG.
    
    // Get the source state and initial values for transition quantities.
    pub fn get_initial(&self) -> CdawgState<Ix> {
        CdawgState {
            state: self.source,
            token: 0,
            start: 0,
            idx: 0,
            end: 0,
            target: self.source,
            length: 0,
        }
    }

    // Transition and track length analogously to the DAWG.
    pub fn transition_and_count(&self, mut cs: CdawgState<Ix>, token: u16) -> CdawgState<Ix> {
        if cs.idx == cs.end { 
            // We are at a state. Analogous to DAWG case.
            let e = self.get_edge_by_token(cs.target, token);
            if let Some(e_val) = e {
                let edge = self.graph.get_edge(e_val);
                let gamma = edge.get_weight().get_span();
                return CdawgState {
                    state: cs.target,
                    token: token,
                    start: gamma.0,
                    idx: gamma.0 + 1,
                    end: usize::min(gamma.1, self.e),
                    target: edge.get_target(),
                    length: cs.length + 1,
                };
            }

            let fail_state = self.graph.get_node(cs.target).get_failure();
            match fail_state {
                Some(q) => {
                    cs.target = q;
                    cs.length = self.graph.get_node(q).get_length();
                    self.transition_and_count(cs, token)
                },
                None => self.get_initial(),
            }
        } else {
            // We are at an edge.
            let cur_token = self.tokens.borrow().get(cs.idx);
            if token == cur_token {
                cs.idx += 1;
                cs.length += 1;
                return cs;
            }

            // Follow implicit failure transitions.
            let fail_state = self.graph.get_node(cs.state).get_failure();
            match fail_state {
                Some(q) => {
                    let q_length = self.graph.get_node(q).get_length();
                    // e is the active edge from fail state corresponding to current edge.
                    let e = self.get_edge_by_token(q, cs.token).unwrap();
                    let edge = self.graph.get_edge(e);
                    let (e_start, e_end) = edge.get_weight().get_span();
                    let progress = cs.idx - cs.start;
                    let new_cs = CdawgState {
                        state: q,
                        token: cs.token,  // Has to be same as e.
                        start: e_start,
                        idx: e_start + progress,
                        end: e_end,
                        target: edge.get_target(),
                        length: q_length + progress as u64,
                    };
                    self.transition_and_count(new_cs, token)
                },
                None => {
                    // We know that cs.state == self.source
                    // Loop until we find a token that exists out of initial state.
                    // In practice, will typically run once.
                    for idx in cs.start + 1..cs.end {
                        let new_token = self.tokens.borrow().get(cs.start + idx);
                        if let Some(e) = self.get_edge_by_token(cs.state, new_token) {
                            let edge = self.graph.get_edge(e);
                            let gamma = edge.get_weight().get_span();
                            let progress = cs.idx - cs.start;
                            let new_cs = CdawgState {
                                state: cs.state,
                                token: new_token,
                                start: gamma.0,
                                idx: gamma.0 + progress - 1,
                                end: gamma.1,
                                target: edge.get_target(),
                                length: (progress - 1) as u64,
                            };
                            return self.transition_and_count(new_cs, token);
                        }
                    }

                    // Well, now we are matching nothing.
                    self.get_initial()
                },
            }
        }
    }

}

#[cfg(test)]
#[allow(unused_variables)]
#[allow(unused_imports)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use graph::memory_backing::disk_backing::disk_vec::DiskVec;

    macro_rules! get_edge {
        // `()` indicates that the macro takes no argument.
        ($c:expr, $q:expr, $w:expr) => {
            $c.graph.get_edge($c.get_edge_by_token($q, $w).unwrap())
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
        let q = cdawg.graph.add_node(DefaultWeight::new(1, Some(cdawg.source), 0));
        cdawg.add_balanced_edge(cdawg.source, q, (1, 1));
        cdawg.add_balanced_edge(q, cdawg.sink, (2, 3));
        cdawg.e = 5;  // Make larger than longest edge.

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
        assert_eq!(start, 2);  // No tokens remaining.

        (state, start) = cdawg.canonize(Some(cdawg.source), (1, 2));
        assert_eq!(state.unwrap().index(), q.index());
        assert_eq!(start, 2);  // One token remaining.

        (state, start) = cdawg.canonize(Some(cdawg.source), (1, 3));
        assert_eq!(state.unwrap().index(), cdawg.sink.index());
        assert_eq!(start, 4);  // No tokens remaining.

        (state, start) = cdawg.canonize(Some(q), (2, 2));
        assert_eq!(state.unwrap().index(), q.index());
        assert_eq!(start, 2);  // One token remaining.

        (state, start) = cdawg.canonize(Some(q), (2, 3));
        assert_eq!(state.unwrap().index(), cdawg.sink.index());
        assert_eq!(start, 4);  // No tokens remaining.
    }

    #[test]
    fn test_check_end_point() {
        // This is 1-indexed!
        let mut cdawg: Cdawg = Cdawg::new(Rc::new(RefCell::new(vec![0, 1, 2])));
        cdawg.add_balanced_edge(cdawg.source, cdawg.sink, (1, 3));
        assert!(cdawg.check_end_point(Some(cdawg.source), to_inenaga((0, 0)), 0));
        assert!(!cdawg.check_end_point(Some(cdawg.source), to_inenaga((0, 0)), 1));
        assert!(cdawg.check_end_point(Some(cdawg.source), to_inenaga((0, 1)), 1));
        assert!(!cdawg.check_end_point(Some(cdawg.source), to_inenaga((0, 1)), 0));

        // From the null state, anything should return true.
        assert!(cdawg.check_end_point(None, to_inenaga((0, 0)), 0));
        assert!(cdawg.check_end_point(None, to_inenaga((0, 0)), 1));
        assert!(cdawg.check_end_point(None, to_inenaga((0, 0)), 2));
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
        cdawg.e += 1;
        cdawg.add_balanced_edge(cdawg.source, cdawg.sink, (1, 3));
        let v = cdawg.split_edge(cdawg.source, to_inenaga((0, 1)));
        
        let idx1 = cdawg.graph.get_node(cdawg.source).get_first_edge();
        let edge1 = cdawg.graph.get_edge(idx1);
        assert_eq!(edge1.get_target().index(), v.index());
        assert_eq!(edge1.get_weight().get_span(), (0, 1));

        let idx2 = cdawg.graph.get_node(v).get_first_edge();
        let edge2 = cdawg.graph.get_edge(idx2);
        assert_eq!(edge2.get_target().index(), cdawg.sink.index());
        assert_eq!(edge2.get_weight().get_span(), (1, 3));

        let target1 = cdawg.extension(cdawg.source, to_inenaga((0, 1)));
        assert_eq!(target1.index(), v.index());
        let target2 = cdawg.extension(target1, to_inenaga((1, 3)));
        assert_eq!(target2.index(), cdawg.sink.index());
    }

    #[test]
    fn test_redirect_edge() {
        let mut cdawg: Cdawg = Cdawg::new(Rc::new(RefCell::new(vec![0, 1, 2])));
        cdawg.add_balanced_edge(cdawg.source, cdawg.sink, (1, 3));
        let target = cdawg.graph.add_node(DefaultWeight::new(0, None, 0));
        cdawg.redirect_edge(cdawg.source, to_inenaga((0, 2)), target);  // Arguments are 1-indexed

        let idx = cdawg.graph.get_node(cdawg.source).get_first_edge();
        let edge: *const crate::graph::avl_graph::Edge<CdawgEdgeWeight> = cdawg.graph.get_edge(idx);
        assert_eq!(edge.get_target().index(), target.index());
        assert_eq!(edge.get_weight().get_span(), (0, 2));  // Graph is 0-indexed
    }

    #[test]
    fn test_separate_node_null() {
        let mut cdawg: Cdawg = Cdawg::new(Rc::new(RefCell::new(vec![0, 1, 2])));
        let c = cdawg.graph.add_node(DefaultWeight::new(1, Some(cdawg.source), 0));
        cdawg.e += 1;
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
        let edge = cdawg.graph.get_edge(cdawg.graph.get_node(cdawg.source).get_first_edge());
        let weight = edge.get_weight();
        assert_eq!(edge.get_target().index(), cdawg.sink.index());
        assert_eq!(cdawg._get_span(weight), (1, 1));  // Wrapper updates value of E
        assert_eq!(cdawg.extension(cdawg.source, to_inenaga((0, 1))).index(), cdawg.sink.index());
        assert_eq!(cdawg.graph.get_node(cdawg.sink).get_failure().unwrap().index(), cdawg.source.index());
        assert_eq!(start, 2);

        // Step 2: co
        (state, start) = cdawg.update(state, start, 2);
        // Correctly has "co" edge instead of "c"?
        let co_idx = cdawg.get_edge_by_token(cdawg.source, c);
        let co_edge = cdawg.graph.get_edge(co_idx.unwrap());
        assert_eq!(cdawg._get_span(co_edge.get_weight()), (1, 2));
        // Correctly has "o" edge?
        let o_idx = cdawg.get_edge_by_token(cdawg.source, o);
        let o_edge = cdawg.graph.get_edge(o_idx.unwrap());
        assert_eq!(cdawg._get_span(o_edge.get_weight()), (2, 2));
        assert_eq!(start, 3);

        // Step 3: coc
        (state, start) = cdawg.update(state, start, 3);
        assert_eq!(cdawg._get_span(co_edge.get_weight()), (1, 3));
        assert_eq!(cdawg._get_span(o_edge.get_weight()), (2, 3));
        assert_eq!(start, 3);  // (3, 3) represents "c"

        // Step 4: coco
        (state, start) = cdawg.update(state, start, 4);
        assert_eq!(cdawg._get_span(co_edge.get_weight()), (1, 4));
        assert_eq!(cdawg._get_span(o_edge.get_weight()), (2, 4));
        assert_eq!(start, 3);  // (3, 4) represents "co"

        // Step 5: cocoa
        (state, start) = cdawg.update(state, start, 5);
        // Verify three edges out of source have the right labels and targets.
        let edge_co = get_edge!(cdawg, cdawg.source, c);
        let edge_o = get_edge!(cdawg, cdawg.source, o);
        let edge_a = get_edge!(cdawg, cdawg.source, a);
        assert_eq!(cdawg._get_span(edge_co.get_weight()), (1, 2));
        assert_eq!(cdawg._get_span(edge_o.get_weight()), (2, 2));
        assert_eq!(cdawg._get_span(edge_a.get_weight()), (5, 5));
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
        assert_eq!(cdawg._get_span(edge_coa.get_weight()), (3, 5));
        assert_eq!(cdawg._get_span(edge_a.get_weight()), (5, 5));

        // Step 6: cocoao
        (state, start) = cdawg.update(state, start, 6);
        // Verify three edges out of source have the right labels and targets.
        let edge_co = get_edge!(cdawg, cdawg.source, c);
        let edge_o = get_edge!(cdawg, cdawg.source, o);
        let edge_ao = get_edge!(cdawg, cdawg.source, a);
        assert_eq!(cdawg._get_span(edge_co.get_weight()), (1, 2));
        assert_eq!(cdawg._get_span(edge_o.get_weight()), (6, 6));
        assert_eq!(cdawg._get_span(edge_ao.get_weight()), (5, 6));
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
        assert_eq!(cdawg._get_span(edge_co_ao.get_weight()), (5, 6));
        assert_eq!(cdawg._get_span(edge_co_coao.get_weight()), (3, 6));
        assert_eq!(edge_co_ao.get_target(), cdawg.sink);
        assert_eq!(edge_co_coao.get_target(), cdawg.sink);
        let edge_o_ao = get_edge!(cdawg, q_o, a);
        let edge_o_coao = get_edge!(cdawg, q_o, c);
        assert_eq!(cdawg._get_span(edge_o_ao.get_weight()), (5, 6));
        assert_eq!(cdawg._get_span(edge_o_coao.get_weight()), (3, 6));
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
        assert_eq!(cdawg._get_span(get_edge!(cdawg, cdawg.source, 0).get_weight()), (1, 1));
        (state, start) = cdawg.update(state, start, 2);
        assert_eq!(get_edge!(cdawg, cdawg.source, 0).get_target(), cdawg.sink);
        assert_eq!(cdawg._get_span(get_edge!(cdawg, cdawg.source, 0).get_weight()), (1, 2));
        (state, start) = cdawg.update(state, start, 3);
        assert_eq!(get_edge!(cdawg, cdawg.source, 0).get_target(), cdawg.sink);
        assert_eq!(cdawg._get_span(get_edge!(cdawg, cdawg.source, 0).get_weight()), (1, 3));

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
        assert_eq!(cdawg._get_span(get_edge!(cdawg, cdawg.source, a).get_weight()), (1, 1));
        assert_eq!(cdawg.graph.n_edges(cdawg.source), 1);

        // 2) ab
        (state, start) = cdawg.update(state, start, 2);
        assert_eq!(get_edge!(cdawg, cdawg.source, a).get_target(), cdawg.sink);
        assert_eq!(cdawg._get_span(get_edge!(cdawg, cdawg.source, a).get_weight()), (1, 2));
        assert_eq!(get_edge!(cdawg, cdawg.source, b).get_target(), cdawg.sink);
        assert_eq!(cdawg._get_span(get_edge!(cdawg, cdawg.source, b).get_weight()), (2, 2));
        assert_eq!(cdawg.graph.n_edges(cdawg.source), 2);

        // 3) abc
        (state, start) = cdawg.update(state, start, 3);
        assert_eq!(get_edge!(cdawg, cdawg.source, a).get_target(), cdawg.sink);
        assert_eq!(cdawg._get_span(get_edge!(cdawg, cdawg.source, a).get_weight()), (1, 3));
        assert_eq!(get_edge!(cdawg, cdawg.source, b).get_target(), cdawg.sink);
        assert_eq!(cdawg._get_span(get_edge!(cdawg, cdawg.source, b).get_weight()), (2, 3));
        assert_eq!(get_edge!(cdawg, cdawg.source, c).get_target(), cdawg.sink);
        assert_eq!(cdawg._get_span(get_edge!(cdawg, cdawg.source, c).get_weight()), (3, 3));
        assert_eq!(cdawg.graph.n_edges(cdawg.source), 3);

        // 4) abca: does not need to split a node
        (state, start) = cdawg.update(state, start, 4);
        assert_eq!(get_edge!(cdawg, cdawg.source, a).get_target(), cdawg.sink);
        assert_eq!(cdawg._get_span(get_edge!(cdawg, cdawg.source, a).get_weight()), (1, 4));
        assert_eq!(get_edge!(cdawg, cdawg.source, b).get_target(), cdawg.sink);
        assert_eq!(cdawg._get_span(get_edge!(cdawg, cdawg.source, b).get_weight()), (2, 4));
        assert_eq!(get_edge!(cdawg, cdawg.source, c).get_target(), cdawg.sink);
        assert_eq!(cdawg._get_span(get_edge!(cdawg, cdawg.source, c).get_weight()), (3, 4));
        assert_eq!(cdawg.graph.n_edges(cdawg.source), 3);

        // 5) abcab: does not need to split a node
        (state, start) = cdawg.update(state, start, 5);
        assert_eq!(get_edge!(cdawg, cdawg.source, a).get_target(), cdawg.sink);
        assert_eq!(cdawg._get_span(get_edge!(cdawg, cdawg.source, a).get_weight()), (1, 5));
        assert_eq!(get_edge!(cdawg, cdawg.source, b).get_target(), cdawg.sink);
        assert_eq!(cdawg._get_span(get_edge!(cdawg, cdawg.source, b).get_weight()), (2, 5));
        assert_eq!(get_edge!(cdawg, cdawg.source, c).get_target(), cdawg.sink);
        assert_eq!(cdawg._get_span(get_edge!(cdawg, cdawg.source, c).get_weight()), (3, 5));
        assert_eq!(cdawg.graph.n_edges(cdawg.source), 3);

        // 6) abcabc: does not need to split a node
        (state, start) = cdawg.update(state, start, 6);
        assert_eq!(get_edge!(cdawg, cdawg.source, a).get_target(), cdawg.sink);
        assert_eq!(cdawg._get_span(get_edge!(cdawg, cdawg.source, a).get_weight()), (1, 6));
        assert_eq!(get_edge!(cdawg, cdawg.source, b).get_target(), cdawg.sink);
        assert_eq!(cdawg._get_span(get_edge!(cdawg, cdawg.source, b).get_weight()), (2, 6));
        assert_eq!(get_edge!(cdawg, cdawg.source, c).get_target(), cdawg.sink);
        assert_eq!(cdawg._get_span(get_edge!(cdawg, cdawg.source, c).get_weight()), (3, 6));
        assert_eq!(cdawg.graph.n_edges(cdawg.source), 3);

        // 7) abcabca: does not need to split a node
        (state, start) = cdawg.update(state, start, 7);
        assert_eq!(get_edge!(cdawg, cdawg.source, a).get_target(), cdawg.sink);
        assert_eq!(cdawg._get_span(get_edge!(cdawg, cdawg.source, a).get_weight()), (1, 7));
        assert_eq!(get_edge!(cdawg, cdawg.source, b).get_target(), cdawg.sink);
        assert_eq!(cdawg._get_span(get_edge!(cdawg, cdawg.source, b).get_weight()), (2, 7));
        assert_eq!(get_edge!(cdawg, cdawg.source, c).get_target(), cdawg.sink);
        assert_eq!(cdawg._get_span(get_edge!(cdawg, cdawg.source, c).get_weight()), (3, 7));
        assert_eq!(cdawg.graph.n_edges(cdawg.source), 3);

        // 8) abcabcab: does not need to split a node
        (state, start) = cdawg.update(state, start, 8);
        assert_eq!(get_edge!(cdawg, cdawg.source, a).get_target(), cdawg.sink);
        assert_eq!(cdawg._get_span(get_edge!(cdawg, cdawg.source, a).get_weight()), (1, 8));
        assert_eq!(get_edge!(cdawg, cdawg.source, b).get_target(), cdawg.sink);
        assert_eq!(cdawg._get_span(get_edge!(cdawg, cdawg.source, b).get_weight()), (2, 8));
        assert_eq!(get_edge!(cdawg, cdawg.source, c).get_target(), cdawg.sink);
        assert_eq!(cdawg._get_span(get_edge!(cdawg, cdawg.source, c).get_weight()), (3, 8));
        assert_eq!(cdawg.graph.n_edges(cdawg.source), 3);
        assert_eq!(start, 4);

        // abcabcaba: where the magic happens! Shown in Figure 13
        (state, start) = cdawg.update(state, start, 9);
        assert_eq!(cdawg.graph.n_edges(cdawg.source), 3);
        assert_eq!(cdawg._get_span(get_edge!(cdawg, cdawg.source, a).get_weight()), (1, 2));
        assert_eq!(cdawg._get_span(get_edge!(cdawg, cdawg.source, b).get_weight()), (2, 2));
        assert_eq!(cdawg._get_span(get_edge!(cdawg, cdawg.source, c).get_weight()), (3, 5));
        let mut qa = get_edge!(cdawg, cdawg.source, a).get_target();
        let mut qb = get_edge!(cdawg, cdawg.source, b).get_target();
        let mut qc = get_edge!(cdawg, cdawg.source, c).get_target();
        assert_eq!(qa, qb);
        assert!(qa != cdawg.source);
        assert!(qc != cdawg.source);
        assert!(qa != qc);
        let q1 = qa;
        let q2 = qc;
        // Check q1 edges.
        assert_eq!(cdawg.graph.n_edges(q1), 2);
        assert_eq!(cdawg._get_span(get_edge!(cdawg, q1, a).get_weight()), (9, 9));
        assert_eq!(cdawg._get_span(get_edge!(cdawg, q1, c).get_weight()), (3, 5));
        assert_eq!(get_edge!(cdawg, q1, a).get_target(), cdawg.sink);
        assert_eq!(get_edge!(cdawg, q1, c).get_target(), q2);
        // Check q2 edges.
        assert_eq!(cdawg.graph.n_edges(q2), 2);
        assert_eq!(cdawg._get_span(get_edge!(cdawg, q1, a).get_weight()), (9, 9));
        assert_eq!(cdawg._get_span(get_edge!(cdawg, q1, c).get_weight()), (3, 5));
        assert_eq!(get_edge!(cdawg, q2, a).get_target(), cdawg.sink);
        assert_eq!(get_edge!(cdawg, q2, c).get_target(), cdawg.sink);
    }

    #[test]
    fn test_sink_length() {
        let (a, b, c) = (0, 1, 2);
        let mut cdawg: Cdawg = Cdawg::new(Rc::new(RefCell::new(vec![a, b, c, a, b, c, a, b, a])));
        let (mut state, mut start) = (cdawg.source, 1);
        (state, start) = cdawg.update(state, start, 1);
        assert_eq!(cdawg._get_length(cdawg.sink), 1);
        (state, start) = cdawg.update(state, start, 2);
        assert_eq!(cdawg._get_length(cdawg.sink), 2);
    }

    type DiskW = DefaultWeight;
    type DiskE = CdawgEdgeWeight<DefaultIx>;
    type DiskCdawg = Cdawg<DiskW, DefaultIx, DiskBacking<DiskW, DiskE, DefaultIx>>;

    #[test]
    fn test_save_load_null() {
        let tmp_dir = tempdir().unwrap();
        let path = tmp_dir.path();

        let tokens: Vec<u16> = vec![10, 11, 12];
        let mb = DiskBacking::new(path);
        let mut cdawg: DiskCdawg = Cdawg::new_mb(Rc::new(RefCell::new(tokens)), mb);
        cdawg.add_balanced_edge(cdawg.source, cdawg.sink, (1, 1));
        cdawg.save(path).unwrap();

        let tokens2: Vec<u16> = vec![10, 11, 12];
        let cdawg2: DiskCdawg = Cdawg::load(Rc::new(RefCell::new(tokens2)), path).unwrap();
        assert_eq!(cdawg2.source, cdawg.source);
        assert_eq!(cdawg2.sink, cdawg.sink);
        assert_eq!(cdawg2.e, cdawg.e);
        assert_eq!(get_edge!(cdawg2, cdawg2.source, 10).get_target(), cdawg.sink);
    }

    #[test]
    fn test_tokens_disk_vec() {
        // Perform step 1 of cocoa on a DiskVec.
        let tmp_dir = tempdir().unwrap();

        let vec: Vec<u16> = vec![0, 1, 2];
        let disk_vec = DiskVec::<u16>::from_vec(vec, tmp_dir.path().join("vec.bin")).unwrap();
        let mut cdawg: Cdawg = Cdawg::new(Rc::new(RefCell::new(disk_vec)));
        let (mut state, mut start) = (cdawg.source, 1);

        // Step 1: c
        (state, start) = cdawg.update(state, start, 1);
        let edge = cdawg.graph.get_edge(cdawg.graph.get_node(cdawg.source).get_first_edge());
        let weight = edge.get_weight();
        assert_eq!(edge.get_target().index(), cdawg.sink.index());
        assert_eq!(cdawg._get_span(weight), (1, 1));  // Wrapper updates value of E
        assert_eq!(cdawg.extension(cdawg.source, to_inenaga((0, 1))).index(), cdawg.sink.index());
        assert_eq!(cdawg.graph.get_node(cdawg.sink).get_failure().unwrap().index(), cdawg.source.index());
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
        for token in vec![a, b, c, a, d].iter() {
            cs = cdawg.transition_and_count(cs, *token);
            lengths.push(cs.length);
        }
        assert_eq!(lengths, vec![1, 2, 3, 3, 0]);
    }

    #[test]
    fn test_transition_and_count_cocoa() {
        let (a, b, c) = (0, 1, 2);
        let train = Rc::new(RefCell::new(vec![a, b, c, a, b, c, a, b, a]));
        let mut cdawg: Cdawg = Cdawg::new(train);
        cdawg.build();

        let mut lengths = Vec::new();
        let mut cs = cdawg.get_initial();
        for token in vec![a, b, a].iter() {
            cs = cdawg.transition_and_count(cs, *token);
            lengths.push(cs.length);
        }
        assert_eq!(lengths, vec![1, 2, 3]);

        lengths = Vec::new();
        cs = cdawg.get_initial();
        for token in vec![a, b, b].iter() {
            cs = cdawg.transition_and_count(cs, *token);
            lengths.push(cs.length);
        }
        assert_eq!(lengths, vec![1, 2, 1]);
    }

}