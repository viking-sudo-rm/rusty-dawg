// Followed the helpful example of: https://github.com/raedwulf/PyCDAWG/blob/master/cdawg.py

mod cdawg_edge_weight;

use graph::{EdgeRef, NodeRef};
use graph::avl_graph::edge::EdgeMutRef;
use graph::avl_graph::node::NodeMutRef;
use graph::avl_graph::AvlGraph;
use graph::indexing::NodeIndex;
use weight::{DefaultWeight, Weight};

use self::cdawg_edge_weight::CdawgEdgeWeight;

pub struct Cdawg {
    tokens: Vec<u16>,
    graph: AvlGraph<DefaultWeight, CdawgEdgeWeight>,
    source: NodeIndex,
    sink: NodeIndex,
}

impl Cdawg {

    pub fn new(tokens: Vec<u16>) -> Self {
        let graph = AvlGraph::new();
        let source = graph.add_node();
        let sink = graph.add_node();
        Self {tokens, graph, source, sink}
    }

    pub fn build(&mut self) {
        for (idx, token) in self.tokens.enumerate() {
            // FIXME: An empty edge... how is this supposed to work?
            let weight = CdawgEdgeWeight::new_full(token, -idx, -idx);
            self.add_balanced_edge(self.source, self.target, weight);
        }
        let mut (state, start) = (self.sink, 0);
        for (idx, token) in self.tokens.enumerate() {
            (state, start) = self.update(state, start, idx);
        }
    }

    fn update(&mut self, mut state: NodeIndex, mut start: usize, end: usize) -> (NodeIndex, usize) {
        let token = self.tokens[start];
        let mut dest: Option<NodeIndex> = None;
        let mut opt_r: Option<NodeIndex> = None;
        let mut opt_old_r: Option<NodeIndex> = None;
        while !self.check_end_point(state, start, end - 1, token) {
            if start <= end - 1 {
                // Implicit case: there is some stuff left.
                let cur_dest = self.get_target(state, start, end - 1);
                if dest == Some(cur_dest) {
                    if let Some(r) = opt_r {
                        self.redirect_edge(state, start, end - 1, r);
                        let fstate = self.graph.get_node(state).get_failure();
                        (state, start) = self.canonize(fstate.unwrap(), start, end - 1);
                    }
                    continue;
                } else {
                    dest = Some(cur_dest);
                    opt_r = Some(self.split_edge(state, start, end - 1));
                }
            } else {
                opt_r = Some(state);
            }

            // This condition should always be hit.
            if let Some(r) = opt_r {
                // FIXME: Original had e[j] here instead of end + 1.
                let weight = CdawgEdgeWeight::new_full(self.tokens[end], end, end + 1);
                self.graph.add_balanced_edge(r, self.sink, weight);
                if let Some(old_r) = opt_old_r {
                    self.graph.get_node_mut(old_r).set_failure(opt_r);
                }
                opt_old_r = opt_r;

                let fstate = self.graph.get_node(state).get_failure().unwrap();
                (state, start) = self.canonize(fstate, start, end - 1);
            }
        }

        if let Some(old_r) = opt_old_r {
            self.graph.get_node_mut(old_r).set_failure(Some(state));
        }
        self.separate_node(state, start, end)
    }

    // This was called extension, but is just following a transition.
    fn get_target(&self, state: NodeIndex, start: usize, end: usize) -> NodeIndex {
        if start > end {
            return state;
        }
        let (_, _, target) = self._get_start_end_target(state, self.tokens[start]);
        target
    }

    // This mutates the weight, but the sort token does not change.
    pub fn redirect_edge(&mut self, state: NodeIndex, start: usize, end: usize, target: NodeIndex) {
        let token = self.tokens[start];
        let edge_idx = self.graph.get_edge_by_weight(state, CdawgEdgeWeight::new(token));
        let edge_ref = self.graph.get_edge(edge_idx.unwrap());
        let (found_start, _) = edge_ref.get_weight().get_span();

        let weight = CdawgEdgeWeight::new_full(token, found_start, found_start + end - start);
        let mut_ref = self.graph.get_edge_mut(edge_idx.unwrap());
        mut_ref.set_weight(weight);
        mut_ref.set_target(target);
    }

    fn split_edge(&mut self, state: NodeIndex, start: usize, end: usize) -> NodeIndex {
        let weight = self.graph.get_node(state).get_weight().clone();
        let new_idx = self.graph.add_node(weight);
        
        // Search for the existing edge to split.
        let token = self.tokens[start];
        let edge_idx = self.graph.get_edge_by_weight(state, CdawgEdgeWeight::new(token));
        let edge_ref = self.graph.get_edge(edge_idx.unwrap());
        let (found_start, found_end) = edge_ref.get_weight().get_span();
        
        // Redirect the original edge to represent the first half.
        let mut_ref = self.graph.get_edge_mut(edge_idx.unwrap());
        mut_ref.set_weight(CdawgEdgeWeight::new_full(token, found_start, found_start + end - start));
        mut_ref.set_target(new_idx);

        // Add a new edge for the second half of the original edge.
        let split_token = self.tokens[found_start + end - start + 1];
        let new_weight = CdawgEdgeWeight::new_full(split_token, found_start + end - start + 1, found_end);
        self.graph.add_balanced_edge(new_idx, edge_ref.get_target(), new_weight);
        new_idx
    }

    fn separate_node(&mut self, mut state: NodeIndex, mut start: usize, end: usize) -> (NodeIndex, usize) {
        let (state1, start1) = self.canonize(state, start, end);
        if start1 <= end {
            // Implicit case: some tokens are left over.
            return (state1, start1);
        }
        let length = self.graph.get_node(state).get_length();
        let length1 = self.graph.get_node(state1).get_length();
        if length1 == length + (end - start + 1) as u64 {
            // Explicit case: all tokens are used up.
            return (state1, start1);
        }

        // Non-solid case: we need to clone a node and insert it along failure path.
        let mut weight = self.graph.get_node(state1).get_weight().clone();
        weight.set_length(length + (end - start + 1) as u64);
        let new_state = self.graph.add_node(weight);
        self.graph.clone_edges(state1, new_state);
        self.graph.get_node_mut(state).set_failure(Some(new_state));
        loop {
            // Replace edges from state to state1 with edges to new_state.
            self._set_start_end_target(state, self.tokens[start], start, end, new_state);
            match self.graph.get_node(state).get_failure() {
                Some(fstate) => {
                    (state, start) = self.canonize(fstate, start, end - 1);
                    if (state1, start1) != self.canonize(state, start, end) {
                        break;
                    }
                }
                None => {break},
            }
        }
        (new_state, end + 1)
    }

    fn canonize(&self, mut state: NodeIndex, mut start: usize, end: usize) -> (NodeIndex, usize) {
        if start > end {
            return (state, start);
        }
        let mut found_start: usize;
        let mut found_end: usize;
        let mut found_state: NodeIndex;

        (found_start, found_end, found_state) = self._get_start_end_target(state, self.tokens[start]);
        while found_end - found_start <= end - start {
            start += found_end - found_start + 1;
            state = found_state;
            if start <= end {
                (found_start, found_end, found_state) = self._get_start_end_target(state, self.tokens[start]);
            }
        }
        (state, start)
    }

    fn check_end_point(&self, state: NodeIndex, start: usize, end: usize, token: u16) -> bool {
        if start <= end {
            let (found_start, _, _) = self._get_start_end_target(state, token);
            return token == self.tokens[found_start + end - start + 1];
        } else {
            return self._graph_has_edge(state, token);
        }
    }

    // Super low-level helper methods.

    fn _get_start_end_target(&self, state: NodeIndex, token: u16) -> (usize, usize, NodeIndex) {
        let search_weight = CdawgEdgeWeight::new(token);
        let edge_idx = self.graph.get_edge_by_weight(state, search_weight);
        let edge_ref = self.graph.get_edge(edge_idx.unwrap());
        let (start, end) = edge_ref.get_weight().get_span();
        let target = edge_ref.get_target();
        (start, end, target)
    }

    fn _set_start_end_target(&mut self, state: NodeIndex, token: u16, start: usize, end: usize, target: NodeIndex) {
        let edge_idx = self.graph.get_edge_by_weight(state, CdawgEdgeWeight::new(token));
        let weight = CdawgEdgeWeight::new_full(token, start, end);
        let mut_ref = self.graph.get_edge_mut(edge_idx.unwrap());
        mut_ref.set_weight(weight);
        mut_ref.set_target(target);
    }

    fn _graph_has_edge(&self, state: NodeIndex, token: u16) -> bool {
        let search_weight = CdawgEdgeWeight::new(token);
        let edge_idx = self.graph.get_edge_by_weight(state, search_weight);
        edge_idx.is_some()
    }

}