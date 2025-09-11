// An immutable version of CDawg

use crate::cdawg::cdawg_state::CdawgState;
use crate::cdawg::comparator::CdawgComparator;
use crate::cdawg::metadata::CdawgMetadata;
use crate::cdawg::{Cdawg, TokenBackingReference};
use crate::graph::array_graph::{ArrayEdgeRef, ArrayGraph, ArrayNodeRef};
use crate::graph::indexing::{DefaultIx, EdgeIndex, IndexType, NodeIndex};
use crate::memory_backing::{
    ArrayMemoryBacking, CacheConfig, DiskBacking, MemoryBacking, RamBacking,
};
use crate::weight::{DefaultWeight, Weight};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

// TODO: Add method to convert icdawg->cdawg
/*
 * In general this class is mainly copied from cdawg (inenga.rs), but I don't think it's smart to
 * built too many abstractions before the structure of what you want to build is solidified -- so
 * I'd rather merge this first and then refactor out the duplicate methods later.
 */
pub struct ImmutableCdawg<W = DefaultWeight, Ix = DefaultIx, Mb = RamBacking<W, (Ix, Ix), Ix>>
where
    Ix: IndexType,
    W: Weight + Clone,
    Mb: ArrayMemoryBacking<W, (Ix, Ix), Ix>,
{
    tokens: TokenBackingReference,
    graph: ArrayGraph<W, (Ix, Ix), Ix, Mb>,
    source: NodeIndex<Ix>,
    sink: NodeIndex<Ix>, // We don't use the sink, but we'd like the be able to convert back to mutable in the future
    end_position: usize, // End position of current document.
}

impl<W, Ix> ImmutableCdawg<W, Ix>
where
    Ix: IndexType,
    W: Weight + Serialize + for<'de> Deserialize<'de> + Clone,
{
    pub fn new<SourceMb: MemoryBacking<W, (Ix, Ix), Ix>>(
        mutable_cdawg: Cdawg<W, Ix, SourceMb>,
    ) -> Self {
        let mb: RamBacking<W, (Ix, Ix), Ix> = RamBacking::default();
        Self::new_mb(
            mutable_cdawg,
            mb,
            CacheConfig {
                edge_cache_size: 0,
                node_cache_size: 0,
            },
        )
    }
}

impl<W, Ix> ImmutableCdawg<W, Ix, DiskBacking<W, (Ix, Ix), Ix>>
where
    Ix: IndexType + Serialize + for<'de> serde::Deserialize<'de>,
    W: Weight + Copy + Serialize + for<'de> Deserialize<'de> + Clone + Default,
    (Ix, Ix): Serialize + for<'de> Deserialize<'de>,
{
    pub fn load<P: AsRef<Path> + Clone + std::fmt::Debug>(
        tokens: TokenBackingReference,
        path: P,
        cache_config: CacheConfig,
    ) -> Result<Self> {
        // Load source/sink from config file if it exists.
        let path2 = path.clone();
        let graph = ArrayGraph::load(path, cache_config)?;

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

impl<W, Ix, Mb> ImmutableCdawg<W, Ix, Mb>
where
    Ix: IndexType,
    W: Weight + Serialize + for<'de> Deserialize<'de> + Clone,
    Mb: ArrayMemoryBacking<W, (Ix, Ix), Ix>,
    Mb::ArrayEdgeRef: Copy,
{
    pub fn new_mb<SourceMb: MemoryBacking<W, (Ix, Ix), Ix>>(
        mutable_cdawg: Cdawg<W, Ix, SourceMb>,
        mb: Mb,
        cache_config: CacheConfig,
    ) -> ImmutableCdawg<W, Ix, Mb> {
        let (tokens, old_graph, source, sink, end_position) = mutable_cdawg.get_data_ownership();
        let graph: ArrayGraph<W, (Ix, Ix), Ix, Mb> =
            ArrayGraph::new_mb(old_graph, mb, cache_config);
        // TODO: Need to make sure the mutable cdawg is deleted here -- maybe implement the drop trait
        Self {
            tokens,
            graph,
            source,
            sink,
            end_position,
        }
    }

    // Get start, end, target associated with an edge.
    // This is 1-indexed for legacy reasons!
    // TODO: Refactor some of the duplicate code between this and cdawg
    pub fn get_start_end_target(&self, edge_idx: EdgeIndex<Ix>) -> (usize, usize, NodeIndex<Ix>) {
        let edge_ref = self.graph.get_edge(edge_idx);
        let target = edge_ref.get_target();
        let span = self.get_span(edge_ref.get_weight(), target);
        // Shift to 1-indexed and retrieve value of end pointer.
        (span.0, span.1, target)
    }

    // Convenience methods.

    pub fn get_graph(&self) -> &ArrayGraph<W, (Ix, Ix), Ix, Mb> {
        &self.graph
    }

    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    pub fn get_source(&self) -> NodeIndex<Ix> {
        self.source
    }

    // Get the Inenaga-indexed span associated with an edge.
    // TODO: Refactor some of the duplicate code between this and cdawg
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
                let e = self.graph.get_edge(edge_idx).get_weight().0.index();
                (start + 1, e + 1) // Adjust both to be 1-indexed.
            }
        }
    }

    // Only well-defined when token is not end-of-text.
    // TODO: Refactor some of the duplicate code between this and cdawg
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
    // TODO: Refactor some of the duplicate code between this and cdawg
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

    // Generalizes failure transition for when we have state + gamma.
    // This is 0-indexed since we use it at inference time.
    // Gamma represents a path of tokens we want to follow from fstate.
    // TODO: Refactor some of the duplicate code between this and cdawg
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

    // Methods for inference with the CDAWG.

    // Get the source state and initial values for transition quantities.
    // TODO: Refactor some of the duplicate code between this and cdawg
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
    // TODO: Refactor some of the duplicate code between this and cdawg
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
    // TODO: Refactor some of the duplicate code between this and cdawg
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

    // TODO: Refactor some of the duplicate code between this and cdawg
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

    // Methods for inference with the CDAWG.

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

    pub fn get_count(&self, state: NodeIndex<Ix>) -> usize {
        self.graph.get_node(state).get_count()
    }

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
}

#[cfg(test)]
#[allow(unused_variables)]
#[allow(unused_imports)]
#[allow(unused_assignments)]
mod tests {
    use crate::cdawg::comparator::CdawgComparator;
    use crate::cdawg::immutable_cdawg::ImmutableCdawg;
    use crate::cdawg::{Cdawg, TopologicalCounter};
    use crate::graph::array_graph::ArrayEdgeRef;
    use crate::graph::indexing::{DefaultIx, IndexType, NodeIndex};
    use crate::memory_backing::{CacheConfig, DiskBacking};
    use crate::weight::DefaultWeight;
    use std::cell::RefCell;
    use std::rc::Rc;
    use tempfile::tempdir;

    macro_rules! get_edge {
        // `()` indicates that the macro takes no argument.
        ($c:expr, $q:expr, $w:expr) => {
            $c.graph.get_edge($c.get_edge_by_token($q, $w).unwrap())
        };
    }

    // Functional tests
    #[test]
    fn test_convert_transition_and_count_abcbca() {
        let (a, b, c, d) = (0, 1, 2, 3);
        let train = Rc::new(RefCell::new(vec![a, b, c, b, c, a]));
        let mut cdawg: Cdawg = Cdawg::new(train);
        cdawg.build();
        let icdawg = ImmutableCdawg::new(cdawg);

        let mut lengths = Vec::new();
        let mut cs = icdawg.get_initial();
        for token in [a, b, c, a, d].iter() {
            cs = icdawg.transition_and_count(cs, *token);
            lengths.push(cs.length);
        }
        assert_eq!(lengths, vec![1, 2, 3, 3, 0]);
    }

    #[test]
    fn test_convert_transition_and_count_abcabcaba() {
        let (a, b, c) = (0, 1, 2);
        let train = Rc::new(RefCell::new(vec![a, b, c, a, b, c, a, b, a]));
        let mut cdawg: Cdawg = Cdawg::new(train);
        cdawg.build();
        let icdawg = ImmutableCdawg::new(cdawg);

        let mut lengths = Vec::new();
        let mut cs = icdawg.get_initial();
        for token in [a, b, a].iter() {
            cs = icdawg.transition_and_count(cs, *token);
            lengths.push(cs.length);
        }
        assert_eq!(lengths, vec![1, 2, 3]);

        lengths = Vec::new();
        cs = icdawg.get_initial();
        for token in [a, b, b].iter() {
            cs = icdawg.transition_and_count(cs, *token);
            lengths.push(cs.length);
        }
        assert_eq!(lengths, vec![1, 2, 1]);
    }

    #[test]
    fn test_convert_transition_and_count_abcbd() {
        // Should test the case where we implicitly fail from a state but canonize not required.
        let (a, b, c, d) = (0, 1, 2, 3);
        let train = Rc::new(RefCell::new(vec![a, b, c, b, d]));
        let mut cdawg: Cdawg = Cdawg::new(train);
        cdawg.build();
        let icdawg = ImmutableCdawg::new(cdawg);

        let mut lengths = Vec::new();
        let mut cs = icdawg.get_initial();
        for token in [a, b, d].iter() {
            cs = icdawg.transition_and_count(cs, *token);
            lengths.push(cs.length);
        }
        assert_eq!(lengths, vec![1, 2, 2]);
    }

    #[test]
    fn test_convert_build_a_end_b_end() {
        let train = Rc::new(RefCell::new(vec![0, u16::MAX, 1, u16::MAX]));
        let mut cdawg: Cdawg = Cdawg::new(train.clone());
        cdawg.build();
        let icdawg = ImmutableCdawg::new(cdawg);

        assert_eq!(icdawg.node_count(), 4); // 3 real nodes + new sink

        // Test the normal edges.
        let edge_a = get_edge!(icdawg, icdawg.source, 0);
        assert_eq!(edge_a.get_target().index(), 1);
        assert_eq!(
            icdawg.get_span(edge_a.get_weight(), edge_a.get_target()),
            (1, 2)
        ); // 1-indexed
        let edge_b = get_edge!(icdawg, icdawg.source, 1);
        assert_eq!(edge_b.get_target().index(), 2);
        assert_eq!(
            icdawg.get_span(edge_b.get_weight(), edge_b.get_target()),
            (3, 4)
        ); // 1-indexed

        // Test the sink edges.
        let cmp0 = CdawgComparator::new(train.clone());
        let doc0 = icdawg.graph.get_edge_by_weight_cmp(
            icdawg.source,
            (DefaultIx::new(1), DefaultIx::new(2)),
            Box::new(cmp0),
        );
        assert_eq!(
            icdawg.graph.get_edge(doc0.unwrap()).get_target(),
            NodeIndex::new(1)
        );
        let cmp1 = CdawgComparator::new(train.clone());
        let doc1 = icdawg.graph.get_edge_by_weight_cmp(
            icdawg.source,
            (DefaultIx::new(3), DefaultIx::new(4)),
            Box::new(cmp1),
        );
        assert_eq!(
            icdawg.graph.get_edge(doc1.unwrap()).get_target(),
            NodeIndex::new(2)
        );

        // Counts just reflect whether a state is sink at this point.
        assert_eq!(icdawg.get_count(NodeIndex::new(0)), 0);
        assert_eq!(icdawg.get_count(NodeIndex::new(1)), 1);
        assert_eq!(icdawg.get_count(NodeIndex::new(2)), 1);
    }

    #[test]
    fn test_convert_get_entropy() {
        // Test counts incrementally.
        let (a, b, c, d) = (0, 1, 2, 3);
        let train = Rc::new(RefCell::new(vec![c, a, b, a, c, u16::MAX]));
        let mut cdawg: Cdawg = Cdawg::new(train);
        cdawg.build();

        let mut counter = TopologicalCounter::new_ram();
        counter.fill_counts(&mut cdawg);
        let icdawg = ImmutableCdawg::new(cdawg);

        let mut entropies = Vec::new();
        let mut cs = icdawg.get_initial();
        for token in [a, b, a, d, c].iter() {
            cs = icdawg.transition_and_count(cs, *token);
            entropies.push(icdawg.get_entropy(cs));
        }

        // The 3rd value is 2 * 1/6 * log2(1/6) + 2 * 2/6 * log2(2/6)

        assert_eq!(entropies[0], 1.);
        assert_eq!(entropies[1], 0.);
        assert_eq!(entropies[2], 0.);
        /* Value didn't match the mutable one exactly -- 1.9182958340544893 vs 1.9182958340544896
         * which I think it just due to the edges being ordered differently allowing some error
         */
        assert!(f64::abs(entropies[3] - 1.9182958340544896) < 0.00000000000001);
        assert_eq!(entropies[4], 1.);
    }

    #[test]
    fn test_convert_get_next_tokens() {
        // Test counts incrementally.
        let (a, b, c, d) = (0, 1, 2, 3);
        let train = Rc::new(RefCell::new(vec![c, a, b, a, c, u16::MAX]));
        let mut cdawg: Cdawg = Cdawg::new(train);
        cdawg.build();

        let mut counter = TopologicalCounter::new_ram();
        counter.fill_counts(&mut cdawg);
        let icdawg = ImmutableCdawg::new(cdawg);

        let mut next_tokens = Vec::new();
        let mut cs = icdawg.get_initial();
        for token in [a, b, a, d, c].iter() {
            cs = icdawg.transition_and_count(cs, *token);
            let mut tokens = icdawg.get_next_tokens(cs);
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

    type DiskW = DefaultWeight;
    type DiskE = (DefaultIx, DefaultIx);
    type DiskCdawg = Cdawg<DiskW, DefaultIx, DiskBacking<DiskW, DiskE, DefaultIx>>;
    type DiskICdawg = ImmutableCdawg<DiskW, DefaultIx, DiskBacking<DiskW, DiskE, DefaultIx>>;

    #[test]
    fn test_save_metadata_load_null() {
        let cdawg_tmp_dir = tempdir().unwrap();
        let cdawg_path = cdawg_tmp_dir.path();
        let icdawg_tmp_dir = tempdir().unwrap();
        let icdawg_path = icdawg_tmp_dir.path();

        let tokens: Vec<u16> = vec![10, 11, 12];
        let cdawg_mb = DiskBacking::new(cdawg_path);
        let mut cdawg: DiskCdawg = Cdawg::new_mb(Rc::new(RefCell::new(tokens)), cdawg_mb);
        cdawg.add_balanced_edge(cdawg.get_source(), NodeIndex::new(1), (1, 1));
        let icdawg_mb = DiskBacking::new(icdawg_path);
        let icdawg = ImmutableCdawg::new_mb(
            cdawg,
            icdawg_mb,
            CacheConfig {
                edge_cache_size: 0,
                node_cache_size: 0,
            },
        );
        icdawg.save_metadata(icdawg_path).unwrap();

        let tokens2: Vec<u16> = vec![10, 11, 12];
        let icdawg2: DiskICdawg = ImmutableCdawg::load(
            Rc::new(RefCell::new(tokens2)),
            icdawg_path,
            CacheConfig::none(),
        )
        .unwrap();
        assert_eq!(icdawg2.source, icdawg.source);
        assert_eq!(icdawg2.sink, icdawg.sink);
        assert_eq!(
            get_edge!(icdawg2, icdawg2.source, 10).get_target(),
            icdawg.sink
        );
    }
}
