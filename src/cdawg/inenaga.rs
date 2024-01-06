// Follows the algorithm from "On-line construction of compact directed acyclic word graphs"
//
// Paper: https://www.sciencedirect.com/science/article/pii/S0166218X04003464
// Python: https://github.com/raedwulf/PyCDAWG/blob/master/cdawg.py 

use std::convert::TryInto;

use graph::{EdgeRef, NodeRef};
use graph::avl_graph::edge::EdgeMutRef;
use graph::avl_graph::node::NodeMutRef;
use graph::avl_graph::AvlGraph;
use graph::indexing::NodeIndex;
use weight::{DefaultWeight, Weight};

use cdawg::cdawg_edge_weight::CdawgEdgeWeight;

// Note that this code uses standard end-inclusive indexing for spans, unlike the paper.
type Span = (usize, usize);

pub struct Cdawg {
    tokens: Vec<u16>,
    graph: AvlGraph<DefaultWeight, CdawgEdgeWeight>,
    source: NodeIndex,
    sink: NodeIndex,
}

impl Cdawg {

    pub fn new(tokens: Vec<u16>) -> Self {
        let mut graph = AvlGraph::new();
        let source = graph.add_node(DefaultWeight::new(0, None, 0));
        let sink = graph.add_node(DefaultWeight::new(tokens.len().try_into().unwrap(), Some(source), 0));
        Self {tokens, graph, source, sink}
    }

    pub fn build(&mut self) {
        // FIXME: The paper adds these, but I don't think we actually need these edges.
        // for (idx, token) in self.tokens.iter().enumerate() {
        //     let weight = CdawgEdgeWeight::new(*token, -idx, -idx);
        //     self.graph.add_balanced_edge(self.source, self.sink, weight);
        // }

        let (mut state, mut start) = (self.sink, 0);
        for idx in 0..self.tokens.len() {
            (state, start) = self.update(state, start, idx);
        }
    }

    fn update(&mut self, mut state: NodeIndex, mut start: usize, end: usize) -> (NodeIndex, usize) {
        let token = self.tokens[start];
        let mut dest: Option<NodeIndex> = None;
        let mut opt_r: Option<NodeIndex> = None;
        let mut opt_old_r: Option<NodeIndex> = None;
        while !self.check_end_point(state, (start, end), token) {
            if start <= end {
                // Implicit case: there is some stuff left.
                let cur_dest = self.get_target(state, (start, end));
                if dest == Some(cur_dest) {
                    if let Some(r) = opt_r {
                        self.redirect_edge(state, (start, end), r);
                        let fstate = self.graph.get_node(state).get_failure();
                        (state, start) = self.canonize(fstate.unwrap(), (start, end));
                    }
                    continue;
                } else {
                    dest = Some(cur_dest);
                    opt_r = Some(self.split_edge(state, (start, end)));
                }
            } else {
                opt_r = Some(state);
            }

            // This condition should always be hit.
            if let Some(r) = opt_r {
                // FIXME: Original had e[j] here instead of end + 1.
                let weight = CdawgEdgeWeight::new(self.tokens[end], end, end + 1);
                self.graph.add_balanced_edge(r, self.sink, weight);
                if let Some(old_r) = opt_old_r {
                    self.graph.get_node_mut(old_r).set_failure(opt_r);
                }
                opt_old_r = opt_r;

                let fstate = self.graph.get_node(state).get_failure().unwrap();
                (state, start) = self.canonize(fstate, (start, end));
            }
        }

        if let Some(old_r) = opt_old_r {
            self.graph.get_node_mut(old_r).set_failure(Some(state));
        }
        self.separate_node(state, (start, end))
    }

    // This was called extension, but is just following a transition.
    fn get_target(&self, state: NodeIndex, gamma: Span) -> NodeIndex {
        let (start, end) = gamma;
        if start >= end {
            return state;
        }
        let (_, _, target) = self._get_start_end_target(state, self.tokens[start]);
        target
    }

    pub fn redirect_edge(&mut self, state: NodeIndex, gamma: Span, target: NodeIndex) {
        let (start, end) = gamma;
        let token = self.tokens[start];
        let edge_idx = self.graph.get_edge_by_weight(state, CdawgEdgeWeight::new_key(token));
        let edge_ref = self.graph.get_edge(edge_idx.unwrap());
        let (found_start, _) = edge_ref.get_weight().get_span();
        let weight = CdawgEdgeWeight::new(token, found_start, found_start + end - start);
        let mut_ref = self.graph.get_edge_mut(edge_idx.unwrap());
        mut_ref.set_weight(weight);
        mut_ref.set_target(target);
    }

    // Split the edge and leave failure transitions unedited.
    fn split_edge(&mut self, q: NodeIndex, gamma: Span) -> NodeIndex {
        // First, create a new node.
        let v = self.graph.add_node(self.graph.get_node(q).get_weight());
        let q_length = self.graph.get_node(q).get_weight().get_length();
        let gamma_length = <usize as TryInto<u64>>::try_into(gamma.1 - gamma.0).unwrap();
        self.graph.get_node_mut(v).set_length(q_length + gamma_length);

        // Next, get the existing edge we're going to split.
        let edge_idx = self.graph.get_edge_by_weight(q, CdawgEdgeWeight::new_key(self.tokens[gamma.0]));
        let edge = self.graph.get_edge(edge_idx.unwrap());
        let (_, end) = edge.get_weight().get_span();
        let target = edge.get_target();

        // Reroute this edge into v.
        let edge_mut = self.graph.get_edge_mut(edge_idx.unwrap());
        edge_mut.set_weight(CdawgEdgeWeight::new(self.tokens[gamma.0], gamma.0, gamma.1));
        edge_mut.set_target(v);

        // Create a new edge from v to the original target.
        let new_weight = CdawgEdgeWeight::new(self.tokens[gamma.1], gamma.1, self.tokens.len());
        self.graph.add_balanced_edge(v, target, new_weight);

        v
    }

    fn separate_node(&mut self, mut state: NodeIndex, gamma: Span) -> (NodeIndex, usize) {
        let (mut start, end) = gamma;
        let (state1, start1) = self.canonize(state, (start, end));
        // Implicit case: some tokens are left over.
        if start1 < end {
            return (state1, start1);
        }

        let length = self.graph.get_node(state).get_length();
        let length1 = self.graph.get_node(state1).get_length();
        // Explicit case: all tokens are used up.
        if length1 == length + (end - start) as u64 {
            return (state1, start1);
        }

        // Non-solid case: we need to clone a node and insert it along failure path.
        // We have start1 == end and length(canon) == length(state) + length(edge)
        let mut weight = self.graph.get_node(state1).get_weight().clone();
        weight.set_length(length + (end - start) as u64);
        let new_state = self.graph.add_node(weight);
        self.graph.clone_edges(state1, new_state);
        self.graph.get_node_mut(state1).set_failure(Some(new_state));

        // Replace edges from state to state1 with edges to new_state.
        loop {
            self._set_start_end_target(state, self.tokens[start], start, end, new_state);
            match self.graph.get_node(state).get_failure() {
                Some(fstate) => {
                    (state, start) = self.canonize(fstate, (start, end));
                    if (state1, start1) != self.canonize(state, (start, end)) {
                        break;
                    }
                }
                None => {break},
            }
        }
        (new_state, end)
    }

    // The point of this is to move through the DAWG until state uses up as much as possible/start is maximized.
    // Returns the last state passed through, and the beginning of the active edge out of it.
    fn canonize(&self, mut state: NodeIndex, gamma: Span) -> (NodeIndex, usize) {
        let (mut start, end) = gamma;
        if start >= end {
            return (state, start);
        }

        let mut found_start: usize;
        let mut found_end: usize;
        let mut found_state: NodeIndex;
        (found_start, found_end, found_state) = self._get_start_end_target(state, self.tokens[start]);
        while found_end - found_start <= end - start {
            start += found_end - found_start;
            state = found_state;
            if start < end {
                (found_start, found_end, found_state) = self._get_start_end_target(state, self.tokens[start]);
            }
        }
        (state, start)
    }

    // Return true if we can keep going; false otherwise.
    fn check_end_point(&self, state: NodeIndex, gamma: Span, token: u16) -> bool {
        let (start, end) = gamma;
        if start < end {
            let wk = self.tokens[start];
            let search_weight = CdawgEdgeWeight::new_key(wk);
            let e = self.graph.get_edge_by_weight(state, search_weight).unwrap();
            let (found_start, _) = self.graph.get_edge(e).get_weight().get_span();
            token == self.tokens[found_start + end - start]
        } else {
            let search_weight = CdawgEdgeWeight::new_key(token);
            let edge_idx = self.graph.get_edge_by_weight(state, search_weight);
            edge_idx.is_some()
        }
    }

    // These helper methods are useful.

    fn _get_start_end_target(&self, state: NodeIndex, token: u16) -> (usize, usize, NodeIndex) {
        let search_weight = CdawgEdgeWeight::new_key(token);
        let edge_idx = self.graph.get_edge_by_weight(state, search_weight);
        let edge_ref = self.graph.get_edge(edge_idx.unwrap());
        let (start, end) = edge_ref.get_weight().get_span();
        let target = edge_ref.get_target();
        (start, end, target)
    }

    fn _set_start_end_target(&mut self, state: NodeIndex, token: u16, start: usize, end: usize, target: NodeIndex) {
        let edge_idx = self.graph.get_edge_by_weight(state, CdawgEdgeWeight::new_key(token));
        let weight = CdawgEdgeWeight::new(token, start, end);
        let mut_ref = self.graph.get_edge_mut(edge_idx.unwrap());
        mut_ref.set_weight(weight);
        mut_ref.set_target(target);
    }

}

#[cfg(test)]
#[allow(unused_variables)]
#[allow(unused_imports)]
mod tests {
    use super::*;

    #[test]
    fn test_canonize() {
        let mut cdawg = Cdawg::new(vec![0, 1, 2]);
        let q = cdawg.graph.add_node(DefaultWeight::new(1, Some(cdawg.source), 0));
        cdawg.graph.add_balanced_edge(cdawg.source, q,  CdawgEdgeWeight::new(0, 0, 1));
        cdawg.graph.add_balanced_edge(q, cdawg.sink,  CdawgEdgeWeight::new(1, 1, 3));

        let (mut state, mut start) = cdawg.canonize(cdawg.source, (0, 0));
        assert_eq!(state.index(), cdawg.source.index());
        assert_eq!(start, 0);  // source, (0, 0)

        (state, start) = cdawg.canonize(cdawg.source, (0, 1));
        assert_eq!(state.index(), q.index());
        assert_eq!(start, 1);  // q, (1, 1)

        (state, start) = cdawg.canonize(cdawg.source, (0, 2));
        assert_eq!(state.index(), q.index());
        assert_eq!(start, 1);  // q, (1, 2)

        (state, start) = cdawg.canonize(cdawg.source, (0, 3));
        assert_eq!(state.index(), cdawg.sink.index());
        assert_eq!(start, 3);  // sink, (3, 3)

        (state, start) = cdawg.canonize(q, (1, 1));
        assert_eq!(state.index(), q.index());
        assert_eq!(start, 1);  // q, (1, 1)

        (state, start) = cdawg.canonize(q, (1, 2));
        assert_eq!(state.index(), q.index());
        assert_eq!(start, 1);  // q, (1, 2)

        (state, start) = cdawg.canonize(q, (1, 3));
        assert_eq!(state.index(), cdawg.sink.index());
        assert_eq!(start, 3);  // sink, (3, 3)
    }

    #[test]
    fn test_check_end_point() {
        let mut cdawg = Cdawg::new(vec![0, 1, 2]);
        let weight = CdawgEdgeWeight::new(0, 0, 3);
        cdawg.graph.add_balanced_edge(cdawg.source, cdawg.sink, weight);
        assert!(cdawg.check_end_point(cdawg.source, (0, 0), 0));
        assert!(!cdawg.check_end_point(cdawg.source, (0, 0), 1));
        assert!(cdawg.check_end_point(cdawg.source, (0, 1), 1));
        assert!(!cdawg.check_end_point(cdawg.source, (0, 1), 0));
    }

    #[test]
    fn test_get_target() {
        let mut cdawg = Cdawg::new(vec![0, 1, 2]);
        let weight = CdawgEdgeWeight::new(0, 0, 3);
        cdawg.graph.add_balanced_edge(cdawg.source, cdawg.sink, weight);
        let target = cdawg.get_target(cdawg.source, (0, 3));
        assert_eq!(target.index(), cdawg.sink.index());
    }

    #[test]
    fn test_split_edge() {
        let mut cdawg = Cdawg::new(vec![0, 1, 2]);
        let weight = CdawgEdgeWeight::new(0, 0, 3);
        cdawg.graph.add_balanced_edge(cdawg.source, cdawg.sink, weight);
        let v = cdawg.split_edge(cdawg.source, (0, 1));
        
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

        let target1 = cdawg.get_target(cdawg.source, (0, 1));
        assert_eq!(target1.index(), v.index());
        let target2 = cdawg.get_target(target1, (1, 3));
        assert_eq!(target2.index(), cdawg.sink.index());
    }

    #[test]
    fn test_redirect_edge() {
        let mut cdawg = Cdawg::new(vec![0, 1, 2]);
        let weight = CdawgEdgeWeight::new(0, 0, 3);
        cdawg.graph.add_balanced_edge(cdawg.source, cdawg.sink, weight);
        let target = cdawg.graph.add_node(DefaultWeight::new(0, None, 0));
        cdawg.redirect_edge(cdawg.source, (0, 2), target);

        let idx = cdawg.graph.get_node(cdawg.source).get_first_edge();
        let edge: *const crate::graph::avl_graph::Edge<CdawgEdgeWeight> = cdawg.graph.get_edge(idx);
        assert_eq!(edge.get_target().index(), target.index());
        assert_eq!(edge.get_weight().token, 0);
        assert_eq!(edge.get_weight().get_span(), (0, 2));
    }

    // #[test]
    // fn test_separate_node() {
    //     let mut cdawg = Cdawg::new(vec![0, 1, 2]);
    //     let c = graph.add_node(DefaultWeight::new(1, Some(cdawg.source), 0));
    //     let co = graph.add_node(DefaultWeight::new(2, Some(c), Some(c)));
    //     cdawg.graph.add_balanced_edge(cdawg.source, c, CdawgEdgeWeight::new(0, 0, 1));
    //     cdawg.graph.add_balanced_edge(c, co, CdawgEdgeWeight::new(1, 1, 2));
    //     cdawg.graph.add_balanced_edge(cdawg.source, co, CdawgEdgeWeight::new(1, 1, 2));
    //     cdawg.graph.add_balanced_edge(co, cdawg.sink, CdawgEdgeWeight::new(2, 2, 3));

    //     let (start, end) = cdawg.separate_node(co, (0, 3));

    //     // FIXME: Not really sure how to test this?
    // }

}