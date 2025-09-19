// Trait defining the common interface for CDAWG implementations

use crate::cdawg::cdawg_state::CdawgState;
use crate::cdawg::comparator::CdawgComparator;
use crate::cdawg::token_backing::TokenBacking;
use crate::cdawg::TokenBackingReference;
use crate::graph::indexing::{EdgeIndex, IndexType, NodeIndex};
use crate::graph::traits::{EdgeRef, NodeRef};
use crate::graph::Graph;
use crate::weight::Weight;
use std::cell::Ref;

/// Common trait for CDAWG implementations (both mutable and immutable)
pub trait ReadableCdawg<N, Ix, G, Node, Edge>
where
    Ix: IndexType,
    N: Weight + Clone,
    G: Graph<N, (Ix, Ix), Ix, Node, Edge>,
    Node: NodeRef<N, Ix> + Copy,
    Edge: EdgeRef<(Ix, Ix), Ix> + Copy,
{
    // Methods that must be implemented by the struct
    fn get_graph(&self) -> &G;
    fn get_source(&self) -> NodeIndex<Ix>;
    fn get_tokens_borrow(&self) -> Ref<'_, dyn TokenBacking<u16>>;
    fn get_tokens_clone(&self) -> TokenBackingReference;
    fn get_end_position(&self) -> usize;

    // Methods implemented in the trait
    fn node_count(&self) -> usize {
        self.get_graph().node_count()
    }

    fn get_count(&self, state: NodeIndex<Ix>) -> usize {
        self.get_graph().get_node(state).get_count()
    }

    // Get the source state and initial values for transition quantities.
    fn get_initial(&self) -> CdawgState<Ix> {
        CdawgState {
            state: self.get_source(),
            edge_start: 0,
            start: 0,
            end: 0,
            target: Some(self.get_source()),
            length: 0,
        }
    }

    // Transition and track length analogously to the DAWG.
    fn transition_and_count(&self, mut cs: CdawgState<Ix>, token: u16) -> CdawgState<Ix> {
        if cs.target.is_none() {
            // Corresponds to the case where we are in the null state after failing.
            self.get_initial()
        } else if cs.start == cs.end {
            // We are at a state. Analogous to DAWG case.
            let e = self.get_edge_by_token(cs.target.unwrap(), token);
            if let Some(e_val) = e {
                let edge = self.get_graph().get_edge(e_val);
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
            let cur_token = self.get_tokens_borrow().get(cs.start);
            if token == cur_token {
                cs.start += 1;
                cs.length += 1;
                return cs;
            }
            let fail_cs = self.implicitly_fail(cs.state, (cs.edge_start, cs.start));
            self.transition_and_count(fail_cs, token)
        }
    }

    fn get_suffix_count(&self, cs: CdawgState<Ix>) -> usize {
        self.get_count(cs.target.unwrap())
    }

    /// Get the entropy of a CDAWG state in bits.
    fn get_entropy(&self, cs: CdawgState<Ix>) -> f64 {
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

    fn get_next_tokens(&self, cs: CdawgState<Ix>) -> Vec<(u16, f64)> {
        let (state, gamma) = cs.get_state_and_gamma();
        if gamma.0 != gamma.1 {
            let token = self.get_tokens_borrow().get(gamma.1);
            return vec![(token, 1.)];
        }

        let q = state.unwrap();
        let denom = self.get_count(q);
        let mut tokens = Vec::new();
        for edge in self.get_graph().edges(q) {
            // let edge_ref = self.graph.get_edge(edge_idx);
            let next_state = edge.get_target();
            let span = self.get_span(edge.get_weight(), next_state);
            let token = self.get_tokens_borrow().get(span.0 - 1); // Shift to 0 indexing.
            let prob = (self.get_count(next_state) as f64) / (denom as f64);
            tokens.push((token, prob));
        }
        tokens
    }

    // Only well-defined when token is not end-of-text.
    fn get_edge_by_token(&self, state: NodeIndex<Ix>, token: u16) -> Option<EdgeIndex<Ix>> {
        if token != u16::MAX {
            let weight = (Ix::new(0), Ix::new(0)); // Doesn't matter.
            let cmp = CdawgComparator::new_with_token(self.get_tokens_clone(), token);
            self.get_graph()
                .get_edge_by_weight_cmp(state, weight, Box::new(cmp))
        } else {
            None
        }
    }

    // Generalizes failure transition for when we have state + gamma.
    // This is 0-indexed since we use it at inference time.
    // Gamma represents a path of tokens we want to follow from fstate.
    fn implicitly_fail(&self, state: NodeIndex<Ix>, gamma: (usize, usize)) -> CdawgState<Ix> {
        let (start, end) = gamma;
        let fstate = self.get_graph().get_node(state).get_failure();

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
                        length: self.get_graph().get_node(q).get_length(),
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
                        length: self.get_graph().get_node(q).get_length() + progress as u64,
                    }
                }
            }
            // We failed from initial state.
            None => CdawgState {
                state: self.get_source(),
                edge_start: 0,
                start: 0,
                end: 0,
                target: None,
                length: 0, // Actually -1 but unsigned.
            },
        }
    }

    // Handle end-of-text tokens correctly.
    fn get_edge_by_token_index(
        &self,
        state: NodeIndex<Ix>,
        token_idx: usize,
    ) -> Option<EdgeIndex<Ix>> {
        let weight = (Ix::new(token_idx), Ix::new(token_idx + 1));
        let token = self.get_tokens_borrow().get(token_idx);
        let cmp = CdawgComparator::new_with_token(self.get_tokens_clone(), token);
        self.get_graph()
            .get_edge_by_weight_cmp(state, weight, Box::new(cmp))
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
                let token = self.get_tokens_borrow().get(start - 1);
                let edge_idx = self.get_edge_by_token(q, token).unwrap();
                (found_start, found_end, found_state) = self.get_start_end_target(edge_idx);
            }
            None => {
                // Changed these to (1, 1) to avoid subtraction overflow issue.
                (found_start, found_end, found_state) = (1, 1, self.get_source());
            }
        }

        while found_end + start <= end + found_start {
            // Written this way to avoid overflow.
            start += found_end + 1 - found_start; // Written this way to avoid overflow.
            state = Some(found_state);
            if start <= end {
                let token = self.get_tokens_borrow().get(start - 1);
                let edge_idx = self.get_edge_by_token(found_state, token).unwrap();
                (found_start, found_end, found_state) = self.get_start_end_target(edge_idx);
            }
        }
        // Map found_start to 1-indexed when we return it.
        (state, start, Some(found_state), found_start, found_end)
    }

    // Get start, end, target associated with an edge.
    // This is 1-indexed for legacy reasons!
    fn get_start_end_target(&self, edge_idx: EdgeIndex<Ix>) -> (usize, usize, NodeIndex<Ix>) {
        let edge_ref = self.get_graph().get_edge(edge_idx);
        let target = edge_ref.get_target();
        let span = self.get_span(edge_ref.get_weight(), target);
        // Shift to 1-indexed and retrieve value of end pointer.
        (span.0, span.1, target)
    }

    // Get the Inenaga-indexed span associated with an edge.
    fn get_span(&self, weight: (Ix, Ix), target: NodeIndex<Ix>) -> (usize, usize) {
        let (start, end) = (weight.0.index(), weight.1.index());
        // Shift to 1-indexed and retrieve value of end pointer.
        if end < Ix::max_value().index() {
            (start + 1, end)
        } else {
            // If there is a self-loop, we are at a different document.
            let edge_idx = self.get_graph().get_node(target).get_first_edge();
            if edge_idx == EdgeIndex::end() {
                // We are in the active document.
                (start + 1, self.get_end_position())
            } else {
                // We are at the sink for a different document.
                let e = self.get_graph().get_edge(edge_idx).get_weight().0.index();
                (start + 1, e + 1) // Adjust both to be 1-indexed.
            }
        }
    }
}
