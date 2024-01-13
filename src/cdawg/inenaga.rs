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

use std::convert::TryInto;

use graph::{EdgeRef, NodeRef};
use graph::avl_graph::edge::EdgeMutRef;
use graph::avl_graph::node::NodeMutRef;
use graph::avl_graph::AvlGraph;
use graph::indexing::NodeIndex;
use weight::{DefaultWeight, Weight};

use cdawg::cdawg_edge_weight::CdawgEdgeWeight;

type Index = usize;
type Span = (Index, Index);
const E: Index = Index::MAX;

pub struct Cdawg {
    tokens: Vec<u16>,
    graph: AvlGraph<DefaultWeight, CdawgEdgeWeight>,
    source: NodeIndex,
    sink: NodeIndex,
    e: usize,
}

impl Cdawg {

    pub fn new(tokens: Vec<u16>) -> Self {
        let mut graph = AvlGraph::new();
        let source = graph.add_node(DefaultWeight::new(0, None, 0));
        let sink = graph.add_node(DefaultWeight::new(tokens.len().try_into().unwrap(), Some(source), 0));
        Self {tokens, graph, source, sink, e: 0}
    }

    pub fn build(&mut self) {
        let (mut state, mut start) = (self.source, 1);
        for idx in 1..self.tokens.len() + 1 {
            self.e = idx;
            (state, start) = self.update(state, start, idx);
        }
    }

    fn update(&mut self,
              in_state: NodeIndex,  // Cannot be null.
              mut start: usize,
              end: usize,) -> (NodeIndex, usize) {
        let token = self.tokens[end - 1];  // Map p back to 0-indexing
        let mut dest: Option<NodeIndex> = None;
        let mut r = NodeIndex::end();
        let mut opt_state: Option<NodeIndex> = Some(in_state);
        let mut opt_r: Option<NodeIndex> = None;
        let mut opt_old_r: Option<NodeIndex> = None;
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
            let weight = self._new_edge_weight(end, E);
            self.graph.add_balanced_edge(r, self.sink, weight);
            
            // 2) Set failure transition.
            if let Some(old_r) = opt_old_r {
                self.graph.get_node_mut(old_r).set_failure(Some(r));
            }
            opt_old_r = Some(r);

            // 3) Update state by canonizing the fstate.
            let old_start = start;
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
    fn extension(&self, state: NodeIndex, gamma: Span) -> NodeIndex {
        let (start, end) = gamma;
        if start > end {
            return state;
        }
        let (_, _, target) = self._get_start_end_target(state, self.tokens[start - 1]);
        target
    }

    // Change the target of the edge coming out of state with path gamma.
    // Note: 1-indexed!
    pub fn redirect_edge(&mut self, state: NodeIndex, gamma: Span, target: NodeIndex) {
        let (start, end) = gamma;
        let token = self.tokens[start - 1];
        let edge_idx = self.graph.get_edge_by_weight(state, CdawgEdgeWeight::new_key(token));
        let edge_ref = self.graph.get_edge(edge_idx.unwrap());
        let (found_start, _) = self._get_span(edge_ref.get_weight());

        let weight = self._new_edge_weight(found_start, found_start + end - start);
        let mut_ref = self.graph.get_edge_mut(edge_idx.unwrap());
        mut_ref.set_weight(weight);
        mut_ref.set_target(target);
    }

    // Split the edge and leave failure transitions unedited.
    fn split_edge(&mut self, q: NodeIndex, gamma: Span) -> NodeIndex {
        // First, create a new node and set it's length.
        let v = self.graph.add_node(self.graph.get_node(q).get_weight());
        let q_length = self.graph.get_node(q).get_weight().get_length();
        let gamma_length = <usize as TryInto<u64>>::try_into(gamma.1 - gamma.0 + 1).unwrap();
        self.graph.get_node_mut(v).set_length(q_length + gamma_length);

        // Next, get the existing edge we're going to split.
        let token = self.tokens[gamma.0 - 1]; // 0-indexed
        let edge_idx = self.graph.get_edge_by_weight(q, CdawgEdgeWeight::new_key(token));
        let edge = self.graph.get_edge(edge_idx.unwrap());
        let (mut start, end) = edge.get_weight().get_span();
        start += 1;  // Map back to Inenaga 1-indexed!
        let target = edge.get_target();

        // Reroute this edge into v.
        let edge_mut = self.graph.get_edge_mut(edge_idx.unwrap());
        edge_mut.set_weight(self._new_edge_weight(start, start + gamma.1 - gamma.0));
        edge_mut.set_target(v);

        // Create a new edge from v to the original target.
        let new_weight = self._new_edge_weight(start + gamma.1 - gamma.0 + 1, end);
        self.graph.add_balanced_edge(v, target, new_weight);

        v
    }

    fn separate_node(&mut self, mut state: Option<NodeIndex>, gamma: Span) -> (NodeIndex, usize) {
        let (mut start, end) = gamma;
        let (opt_state1, mut start1) = self.canonize(state, (start, end));
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
        if length1 == length + (start + 1 - end) as i64 {
            return (state1, start1);
        }

        // Non-solid, explicit case: we are at a node and need to clone it.
        let mut weight = self.graph.get_node(state1).get_weight().clone();
        weight.set_length((length + (end - start + 1) as i64) as u64);
        let new_state = self.graph.add_node(weight);
        self.graph.clone_edges(state1, new_state);
        self.graph.get_node_mut(state1).set_failure(Some(new_state));

        // Replace edges from state to state1 with edges to new_state.
        // We know that state is non-null here.
        loop {
            // Replace edge with (start, end) but change indexing!
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
    fn canonize(&self, mut state: Option<NodeIndex>, gamma: Span) -> (Option<NodeIndex>, usize) {
        let (mut start, mut end) = gamma;
        if start > end {
            // Means we are at a state.
            return (state, start);
        }

        let mut found_start: usize;
        let mut found_end: usize;
        let mut found_state: NodeIndex;
        match state {
            Some(q) => {
                let token = self.tokens[start - 1];
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
                let token = self.tokens[start - 1];
                (found_start, found_end, found_state) = self._get_start_end_target(found_state, token);
            }
        }
        (state, start)
    }

    // Return true if we can keep going; false otherwise.
    // 1-indexed!
    fn check_end_point(&self, state: Option<NodeIndex>, gamma: Span, token: u16) -> bool {
        let (start, end) = gamma;
        if start <= end {
            let wk = self.tokens[start - 1];
            let search_weight = CdawgEdgeWeight::new_key(wk);
            let e = self.graph.get_edge_by_weight(state.unwrap(), search_weight).unwrap();
            let (found_start, _) = self._get_span(self.graph.get_edge(e).get_weight());
            token == self.tokens[found_start + end - start]  // No +1 because 0-indexed.
        } else {
            match state {
                Some(phi) => {
                    let search_weight = CdawgEdgeWeight::new_key(token);
                    let edge_idx = self.graph.get_edge_by_weight(phi, search_weight);
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
    fn _new_edge_weight(&mut self, start: usize, end: usize) -> CdawgEdgeWeight {
        CdawgEdgeWeight::new(
            self.tokens[start - 1],  // Map start to 0-indexed
            start - 1,  // Map start to 0-indexed
            end,  // Keep end as 1-indexed
        )
    }

    // Get the Inenaga-indexed span associated with an edge.
    // Maybe make this a macro?
    fn _get_span(&self, weight: CdawgEdgeWeight) -> Span {
        let (start, end) = weight.get_span();
        // Shift to 1-indexed and retrieve value of end pointer.
        (start + 1, Index::min(end, self.e))
    }

    fn _get_start_end_target(&self, state: NodeIndex, token: u16) -> (usize, usize, NodeIndex) {
        let search_weight = CdawgEdgeWeight::new_key(token);
        let edge_idx = self.graph.get_edge_by_weight(state, search_weight);
        let edge_ref = self.graph.get_edge(edge_idx.unwrap());
        let (start, end) = edge_ref.get_weight().get_span();
        let target = edge_ref.get_target();
        // Shift to 1-indexed and retrieve value of end pointer.
        (start + 1, Index::min(end, self.e), target)
    }

    fn _set_start_end_target(&mut self, state: NodeIndex, start: usize, end: usize, target: NodeIndex) {
        let token = self.tokens[start - 1];  // Potential double retrieval of token here.
        let edge_idx = self.graph.get_edge_by_weight(state, CdawgEdgeWeight::new_key(token));
        let mut_ref = self.graph.get_edge_mut(edge_idx.unwrap());
        mut_ref.set_weight(self._new_edge_weight(start, end));
        mut_ref.set_target(target);
    }

}

pub fn to_inenaga(gamma: Span) -> Span {
    (gamma.0 + 1, gamma.1)
}

pub fn to_native(gamma: Span) -> Span {
    (gamma.0 - 1, gamma.1)
}

#[cfg(test)]
#[allow(unused_variables)]
#[allow(unused_imports)]
mod tests {
    use super::*;

    macro_rules! _get_edge {
        // `()` indicates that the macro takes no argument.
        ($c:expr, $q:expr, $w:expr) => {
            $c.graph.get_edge($c.graph.get_edge_by_weight($q, CdawgEdgeWeight::new_key($w)).unwrap())
        };
    }

    #[test]
    fn test_canonize() {
        // Test canonize, which uses 1-indexing!
        let mut cdawg = Cdawg::new(vec![0, 1, 2]);
        let q = cdawg.graph.add_node(DefaultWeight::new(1, Some(cdawg.source), 0));
        let weight1 = cdawg._new_edge_weight(1, 1);
        cdawg.graph.add_balanced_edge(cdawg.source, q, weight1);
        let weight2 = cdawg._new_edge_weight(2, 3);
        cdawg.graph.add_balanced_edge(q, cdawg.sink, weight2);
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
        let mut cdawg = Cdawg::new(vec![0, 1, 2]);
        let weight = CdawgEdgeWeight::new(0, 0, 3);
        cdawg.graph.add_balanced_edge(cdawg.source, cdawg.sink, weight);
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
        let mut cdawg = Cdawg::new(vec![0, 1, 2]);
        let weight = CdawgEdgeWeight::new(0, 0, 3);
        cdawg.graph.add_balanced_edge(cdawg.source, cdawg.sink, weight);
        let target = cdawg.extension(cdawg.source, to_inenaga((0, 3)));
        assert_eq!(target.index(), cdawg.sink.index());
    }

    #[test]
    fn test_split_edge() {
        let mut cdawg = Cdawg::new(vec![0, 1, 2]);
        let weight = CdawgEdgeWeight::new(0, 0, 3);
        cdawg.graph.add_balanced_edge(cdawg.source, cdawg.sink, weight);
        let v = cdawg.split_edge(cdawg.source, to_inenaga((0, 1)));
        
        let idx1 = cdawg.graph.get_node(cdawg.source).get_first_edge();
        let edge1 = cdawg.graph.get_edge(idx1);
        assert_eq!(edge1.get_target().index(), v.index());
        assert_eq!(edge1.get_weight().token, 0);
        assert_eq!(edge1.get_weight().get_span(), (0, 1));

        let idx2 = cdawg.graph.get_node(v).get_first_edge();
        let edge2 = cdawg.graph.get_edge(idx2);
        assert_eq!(edge2.get_target().index(), cdawg.sink.index());
        assert_eq!(edge2.get_weight().token, 1);
        assert_eq!(edge2.get_weight().get_span(), (1, 3));

        let target1 = cdawg.extension(cdawg.source, to_inenaga((0, 1)));
        assert_eq!(target1.index(), v.index());
        let target2 = cdawg.extension(target1, to_inenaga((1, 3)));
        assert_eq!(target2.index(), cdawg.sink.index());
    }

    #[test]
    fn test_redirect_edge() {
        let mut cdawg = Cdawg::new(vec![0, 1, 2]);
        let weight = CdawgEdgeWeight::new(0, 0, 3);
        cdawg.graph.add_balanced_edge(cdawg.source, cdawg.sink, weight);
        let target = cdawg.graph.add_node(DefaultWeight::new(0, None, 0));
        cdawg.redirect_edge(cdawg.source, to_inenaga((0, 2)), target);  // Arguments are 1-indexed

        let idx = cdawg.graph.get_node(cdawg.source).get_first_edge();
        let edge: *const crate::graph::avl_graph::Edge<CdawgEdgeWeight> = cdawg.graph.get_edge(idx);
        assert_eq!(edge.get_target().index(), target.index());
        assert_eq!(edge.get_weight().token, 0);
        assert_eq!(edge.get_weight().get_span(), (0, 2));  // Graph is 0-indexed
    }

    #[test]
    fn test_separate_node_null() {
        let mut cdawg = Cdawg::new(vec![0, 1, 2]);
        let c = cdawg.graph.add_node(DefaultWeight::new(1, Some(cdawg.source), 0));
        cdawg.graph.add_balanced_edge(cdawg.source, c, CdawgEdgeWeight::new(0, 0, 1));

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
        let mut cdawg = Cdawg::new(vec![c, o, c, o, a, o]);
        let (mut state, mut start) = (cdawg.source, 1);

        // Step 1: c
        cdawg.e += 1;
        (state, start) = cdawg.update(state, start, cdawg.e);
        let edge = cdawg.graph.get_edge(cdawg.graph.get_node(cdawg.source).get_first_edge());
        let weight = edge.get_weight();
        assert_eq!(edge.get_target().index(), cdawg.sink.index());
        assert_eq!(weight.token, c);
        assert_eq!(cdawg._get_span(weight), (1, 1));  // Wrapper updates value of E
        assert_eq!(cdawg.extension(cdawg.source, to_inenaga((0, 1))).index(), cdawg.sink.index());
        assert_eq!(cdawg.graph.get_node(cdawg.sink).get_failure().unwrap().index(), cdawg.source.index());
        assert_eq!(start, 2);

        // Step 2: co
        cdawg.e += 1;
        (state, start) = cdawg.update(state, start, cdawg.e);
        // Correctly has "o" edge?
        let o_idx = cdawg.graph.get_edge_by_weight(cdawg.source, CdawgEdgeWeight::new_key(o));
        let o_edge = cdawg.graph.get_edge(o_idx.unwrap());
        assert_eq!(o_edge.get_weight().token, o);
        assert_eq!(cdawg._get_span(o_edge.get_weight()), (2, 2));
        // Correctly has "co" edge instead of "c"?
        let co_idx = cdawg.graph.get_edge_by_weight(cdawg.source, CdawgEdgeWeight::new_key(c));
        let co_edge = cdawg.graph.get_edge(co_idx.unwrap());
        assert_eq!(co_edge.get_weight().token, c);
        assert_eq!(cdawg._get_span(co_edge.get_weight()), (1, 2));
        assert_eq!(start, 3);

        // Step 3: coc
        cdawg.e += 1;
        (state, start) = cdawg.update(state, start, cdawg.e);
        assert_eq!(co_edge.get_weight().token, c);
        assert_eq!(cdawg._get_span(co_edge.get_weight()), (1, 3));
        assert_eq!(o_edge.get_weight().token, o);
        assert_eq!(cdawg._get_span(o_edge.get_weight()), (2, 3));
        assert_eq!(start, 3);  // (3, 3) represents "c"

        // Step 4: coco
        cdawg.e += 1;
        (state, start) = cdawg.update(state, start, cdawg.e);
        assert_eq!(co_edge.get_weight().token, c);
        assert_eq!(cdawg._get_span(co_edge.get_weight()), (1, 4));
        assert_eq!(o_edge.get_weight().token, o);
        assert_eq!(cdawg._get_span(o_edge.get_weight()), (2, 4));
        assert_eq!(start, 3);  // (3, 4) represents "co"

        // Step 5: cocoa
        cdawg.e += 1;
        (state, start) = cdawg.update(state, start, cdawg.e);
        // Verify three edges out of source have the right labels and targets.
        let edge_co = _get_edge!(cdawg, cdawg.source, c);
        let edge_o = _get_edge!(cdawg, cdawg.source, o);
        let edge_a = _get_edge!(cdawg, cdawg.source, a);
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
        let edge_coa = _get_edge!(cdawg, q, c);
        let edge_a2 = _get_edge!(cdawg, q, a);
        assert_eq!(cdawg._get_span(edge_coa.get_weight()), (3, 5));
        assert_eq!(cdawg._get_span(edge_a.get_weight()), (5, 5));

        // Step 6: cocoao
        cdawg.e += 1;
        (state, start) = cdawg.update(state, start, cdawg.e);
        // Verify three edges out of source have the right labels and targets.
        let edge_co = _get_edge!(cdawg, cdawg.source, c);
        let edge_o = _get_edge!(cdawg, cdawg.source, o);
        let edge_ao = _get_edge!(cdawg, cdawg.source, a);
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
        let edge_co_ao = _get_edge!(cdawg, q_co, a);
        let edge_co_coao = _get_edge!(cdawg, q_co, c);
        assert_eq!(cdawg._get_span(edge_co_ao.get_weight()), (5, 6));
        assert_eq!(cdawg._get_span(edge_co_coao.get_weight()), (3, 6));
        assert_eq!(edge_co_ao.get_target(), cdawg.sink);
        assert_eq!(edge_co_coao.get_target(), cdawg.sink);
        let edge_o_ao = _get_edge!(cdawg, q_o, a);
        let edge_o_coao = _get_edge!(cdawg, q_o, c);
        assert_eq!(cdawg._get_span(edge_o_ao.get_weight()), (5, 6));
        assert_eq!(cdawg._get_span(edge_o_coao.get_weight()), (3, 6));
        assert_eq!(edge_o_ao.get_target(), cdawg.sink);
        assert_eq!(edge_o_coao.get_target(), cdawg.sink);
    }

}