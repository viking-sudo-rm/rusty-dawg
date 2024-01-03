// Followed the helpful example of: https://github.com/raedwulf/PyCDAWG/blob/master/cdawg.py

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
        // for idx in 0..self.tokens.len() {
        //     // FIXME: An empty edge... how is this supposed to work?
        //     let weight = CdawgEdgeWeight::new_full(token, -idx, -idx);
        //     self.add_balanced_edge(self.source, self.target, weight);
        // }
        // let mut (state, start) = (self.sink, 0);
        // for (idx, token) in self.tokens.enumerate() {
        //     (state, start) = self.update(state, start, idx);
        // }
    }

    pub fn slow_find(&self, idx: usize, mut p: NodeIndex) -> (NodeIndex, Span) {
        let mut edge: Option<Span> = None;  // Current edge label, represented as span.
        let mut edge_pos = 0;  // Position along current edge.
        let mut target: Option<NodeIndex> = None;  // Target of current edge.

        for token in &self.tokens[idx..] {
            match edge {
                Some(span) => {
                    let edge_token = self.tokens[span.0 + edge_pos];
                    if *token == edge_token {
                        edge_pos += 1;
                        // Check if we have completed the edge.
                        if span.0 + edge_pos == span.1 {
                            p = target.unwrap();
                            edge = None;
                        }
                    } else {
                        return (p, (span.0, span.0 + edge_pos));
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
                            // We hit a state and there is no edge out.
                            return (p, (0, 0))
                        }
                    };
                },
            };
        }
        
        // Not sure if both cases are reachable.
        match edge {
            Some(span) => (p, (span.0, span.0 + edge_pos)),
            None => (p, (0, 0)),
        }
        
    }

    fn get_edge_idx(&self, state: NodeIndex, token: u16) -> Option<EdgeIndex> {
        let search_weight = CdawgEdgeWeight::new(token);
        self.graph.get_edge_by_weight(state, search_weight)
    }

}