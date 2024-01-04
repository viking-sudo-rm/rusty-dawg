// Follows the algorithm from "On Compact Directed Acyclic Word Graphs"
// 
// Other implementations use a different version of algo, e.g.:
// https://github.com/raedwulf/PyCDAWG/blob/master/cdawg.py

mod cdawg_edge_weight;

use std::convert::TryInto;

use graph::{EdgeRef, NodeRef};
use graph::avl_graph::edge::EdgeMutRef;
use graph::avl_graph::node::NodeMutRef;
use graph::avl_graph::AvlGraph;
use graph::indexing::{NodeIndex, EdgeIndex};
use weight::{DefaultWeight, Weight};

use self::cdawg_edge_weight::CdawgEdgeWeight;

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
        let mut p = self.source;
        for idx in 0..self.tokens.len() {
            let (q, gamma, end) = self.slow_find(idx, p);
            let tail = self.new_span(end, self.tokens.len());

            if gamma.0 == gamma.1 {
                // Extra tokens matched on edge are empty.
                self.graph.add_balanced_edge(q, self.sink, tail);
                self.graph.get_node_mut(self.sink).set_failure(Some(q));
                p = if q != self.source {
                    self.graph.get_node(q).get_failure().unwrap()
                } else {self.source};
            } else {
                let v = self.split_edge(q, gamma);
                self.graph.add_balanced_edge(v, self.sink, tail);
                self.graph.get_node_mut(self.sink).set_failure(Some(v));
                let (r, _) = self.fast_find(idx, p, gamma);
                p = r;
            }
        }
    }

    pub fn slow_find(&self, idx: usize, mut p: NodeIndex) -> (NodeIndex, Span, usize) {
        let mut edge: Option<Span> = None;  // Current edge label, represented as span.
        let mut edge_pos = 0;  // Position along current edge.
        let mut target: Option<NodeIndex> = None;  // Target of current edge.
        let mut end = idx;

        for token in &self.tokens[idx..] {
            match edge {
                Some(span) => {
                    let edge_token = self.tokens[span.0 + edge_pos];
                    if *token != edge_token {
                        // The next token on the edge does not match.
                        return (p, (span.0, span.0 + edge_pos), end);
                    }

                    edge_pos += 1;
                    // Check if we have completed the edge.
                    if span.0 + edge_pos == span.1 {
                        p = target.unwrap();
                        edge = None;
                    }
                },
                None => {
                    // We are on a state, rather than along an edge.
                    match self.get_edge_idx(p, *token) {
                        Some(edge_idx) => {
                            let e = self.graph.get_edge(edge_idx);
                            let span = e.get_weight().get_span();
                            edge = Some(span);
                            edge_pos = 1;
                            target = Some(e.get_target());
                            // Check if we have already completed the edge.
                            if span.0 + 1 == span.1 {
                                p = target.unwrap();
                                edge = None;
                            }
                        },
                        None => {
                            // The right edge out of the state does not exist.
                            return (p, (0, 0), end);
                        },
                    };
                },
            };
            end += 1;
        }
        
        // Not sure if both cases are reachable.
        match edge {
            Some(span) => (p, (span.0, span.0 + edge_pos), end),
            None => (p, (0, 0), end),
        }
    }

    fn fast_find(&mut self, idx: usize, mut p: NodeIndex, gamma: Span) -> (NodeIndex, Span) {
        // TODO: Implement FastFind
        (self.source, (0, 0))
    }

    fn split_edge(&mut self, q: NodeIndex, gamma: Span) -> NodeIndex {
        // Split the edge and leave failure transitions unedited.

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

    fn get_edge_idx(&self, state: NodeIndex, token: u16) -> Option<EdgeIndex> {
        let search_weight = CdawgEdgeWeight::new_key(token);
        self.graph.get_edge_by_weight(state, search_weight)
    }

    fn new_span(&self, start: usize, end: usize) -> CdawgEdgeWeight {
        CdawgEdgeWeight::new(self.tokens[start], start, end)
    }
}

#[cfg(test)]
#[allow(unused_variables)]
#[allow(unused_imports)]
mod tests {
    use super::*;

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
    }

    fn test_slow_find_source() {
        let mut cdawg = Cdawg::new(vec![0, 0, 1]);
        let weight = CdawgEdgeWeight::new(0, 0, 3);
        cdawg.graph.add_balanced_edge(cdawg.source, cdawg.sink, weight);
        let (q, gamma, end) = cdawg.slow_find(1, cdawg.source);
        assert_eq!(q.index(), cdawg.source.index());
        assert_eq!(gamma, (0, 1));
        assert_eq!(end, 1);
    }

    fn test_slow_find_node() {
        let mut cdawg = Cdawg::new(vec![0, 0, 0]);
        let v = cdawg.graph.add_node(DefaultWeight::new(1, Some(cdawg.source), 0));
        cdawg.graph.get_node_mut(cdawg.sink).set_failure(Some(v));
        cdawg.graph.add_balanced_edge(cdawg.source, v, CdawgEdgeWeight::new(0, 0, 1));
        cdawg.graph.add_balanced_edge(v, cdawg.sink, CdawgEdgeWeight::new(0, 1, 3));

        let (q, gamma, end) = cdawg.slow_find(1, cdawg.source);
        assert_eq!(q.index(), v.index());
        assert_eq!(gamma, (1, 2));
        assert_eq!(end, 2);
    }

}