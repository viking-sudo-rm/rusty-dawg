// An immutable version of CDawg

use crate::cdawg::cdawg_state::CdawgState;
use crate::cdawg::metadata::CdawgMetadata;
use crate::cdawg::readable_cdawg::ReadableCdawg;
use crate::cdawg::token_backing::TokenBacking;
use crate::cdawg::{Cdawg, TokenBackingReference};
use crate::graph::array_graph::ArrayGraph;
use crate::graph::indexing::{DefaultIx, EdgeIndex, IndexType, NodeIndex};
use crate::memory_backing::{
    ArrayMemoryBacking, CacheConfig, DiskBacking, MemoryBacking, RamBacking,
};
use crate::weight::{DefaultWeight, Weight};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::cell::Ref;
use std::path::Path;

// TODO: Add method to convert icdawg->cdawg
/*
 * In general this class is mainly copied from cdawg (inenaga), but I don't think it's smart to
 * built too many abstractions before the structure of what you want to build is solidified -- so
 * I'd rather merge this first and then refactor out the duplicate methods later.
 */
pub struct ArrayCdawg<N = DefaultWeight, Ix = DefaultIx, Mb = RamBacking<N, (Ix, Ix), Ix>>
where
    Ix: IndexType,
    N: Weight + Clone,
    Mb: ArrayMemoryBacking<N, (Ix, Ix), Ix>,
{
    tokens: TokenBackingReference,
    graph: ArrayGraph<N, (Ix, Ix), Ix, Mb>,
    source: NodeIndex<Ix>,
    sink: NodeIndex<Ix>, // We don't use the sink, but we'd like the be able to convert back to mutable in the future
    end_position: usize, // End position of current document.
}

impl<N, Ix> ArrayCdawg<N, Ix>
where
    Ix: IndexType,
    N: Weight + Serialize + for<'de> Deserialize<'de> + Clone + Copy,
{
    pub fn new<SourceMb: MemoryBacking<N, (Ix, Ix), Ix>>(
        mutable_cdawg: Cdawg<N, Ix, SourceMb>,
    ) -> Self {
        let mb: RamBacking<N, (Ix, Ix), Ix> = RamBacking::default();
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

impl<N, Ix> ArrayCdawg<N, Ix, DiskBacking<N, (Ix, Ix), Ix>>
where
    Ix: IndexType + Serialize + for<'de> serde::Deserialize<'de>,
    N: Weight + Copy + Serialize + for<'de> Deserialize<'de> + Clone + Default,
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

impl<N, Ix, Mb> ArrayCdawg<N, Ix, Mb>
where
    Ix: IndexType,
    N: Weight + Serialize + for<'de> Deserialize<'de> + Clone + Copy,
    Mb: ArrayMemoryBacking<N, (Ix, Ix), Ix>,
    Mb::ArrayNodeRef: Copy,
    Mb::ArrayEdgeRef: Copy,
{
    pub fn new_mb<SourceMb: MemoryBacking<N, (Ix, Ix), Ix>>(
        mutable_cdawg: Cdawg<N, Ix, SourceMb>,
        mb: Mb,
        cache_config: CacheConfig,
    ) -> ArrayCdawg<N, Ix, Mb> {
        let (tokens, old_graph, source, sink, end_position) = mutable_cdawg.get_data_ownership();
        let graph: ArrayGraph<N, (Ix, Ix), Ix, Mb> =
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

    // Local calls to immutable cdawg functions for convenience
    #[allow(clippy::type_complexity)]
    fn as_immutable_cdawg(
        &self,
    ) -> &dyn ReadableCdawg<
        N,
        Ix,
        ArrayGraph<N, (Ix, Ix), Ix, Mb>,
        Mb::ArrayNodeRef,
        Mb::ArrayEdgeRef,
    > {
        self
    }
    pub fn node_count(&self) -> usize {
        self.as_immutable_cdawg().node_count()
    }
    pub fn get_count(&self, state: NodeIndex<Ix>) -> usize {
        self.as_immutable_cdawg().get_count(state)
    }
    pub fn get_initial(&self) -> CdawgState<Ix> {
        self.as_immutable_cdawg().get_initial()
    }
    pub fn transition_and_count(&self, cs: CdawgState<Ix>, token: u16) -> CdawgState<Ix> {
        self.as_immutable_cdawg().transition_and_count(cs, token)
    }

    pub fn get_suffix_count(&self, cs: CdawgState<Ix>) -> usize {
        self.as_immutable_cdawg().get_suffix_count(cs)
    }
    pub fn get_entropy(&self, cs: CdawgState<Ix>) -> f64 {
        self.as_immutable_cdawg().get_entropy(cs)
    }
    pub fn get_next_tokens(&self, cs: CdawgState<Ix>) -> Vec<(u16, f64)> {
        self.as_immutable_cdawg().get_next_tokens(cs)
    }
    pub fn get_edge_by_token(&self, state: NodeIndex<Ix>, token: u16) -> Option<EdgeIndex<Ix>> {
        self.as_immutable_cdawg().get_edge_by_token(state, token)
    }
    pub fn implicitly_fail(&self, state: NodeIndex<Ix>, gamma: (usize, usize)) -> CdawgState<Ix> {
        self.as_immutable_cdawg().implicitly_fail(state, gamma)
    }
    pub fn get_edge_by_token_index(
        &self,
        state: NodeIndex<Ix>,
        token_idx: usize,
    ) -> Option<EdgeIndex<Ix>> {
        self.as_immutable_cdawg()
            .get_edge_by_token_index(state, token_idx)
    }

    // Inference-time version of canonize. Crucially:
    //   1. returns target state.
    pub fn inference_canonize(
        &self,
        state: Option<NodeIndex<Ix>>,
        gamma: (usize, usize),
    ) -> (
        Option<NodeIndex<Ix>>,
        usize,
        Option<NodeIndex<Ix>>,
        usize,
        usize,
    ) {
        self.as_immutable_cdawg().inference_canonize(state, gamma)
    }
    // Get start, end, target associated with an edge.
    // This is 1-indexed for legacy reasons!
    pub fn get_start_end_target(&self, edge_idx: EdgeIndex<Ix>) -> (usize, usize, NodeIndex<Ix>) {
        self.as_immutable_cdawg().get_start_end_target(edge_idx)
    }
    pub fn get_span(&self, weight: (Ix, Ix), target: NodeIndex<Ix>) -> (usize, usize) {
        self.as_immutable_cdawg().get_span(weight, target)
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

    pub fn get_graph(&self) -> &ArrayGraph<N, (Ix, Ix), Ix, Mb> {
        &self.graph
    }
}

// Implement the ImmutableCdawg trait for ArrayCdawg
impl<N, Ix, Mb>
    ReadableCdawg<N, Ix, ArrayGraph<N, (Ix, Ix), Ix, Mb>, Mb::ArrayNodeRef, Mb::ArrayEdgeRef>
    for ArrayCdawg<N, Ix, Mb>
where
    Ix: IndexType,
    N: Weight + Serialize + for<'de> Deserialize<'de> + Clone,
    Mb: ArrayMemoryBacking<N, (Ix, Ix), Ix>,
    Mb::ArrayEdgeRef: Copy,
    Mb::ArrayNodeRef: Copy,
{
    fn get_graph(&self) -> &ArrayGraph<N, (Ix, Ix), Ix, Mb> {
        &self.graph
    }
    fn get_source(&self) -> NodeIndex<Ix> {
        self.source
    }
    fn get_tokens_borrow(&self) -> Ref<'_, dyn TokenBacking<u16>> {
        self.tokens.borrow()
    }

    fn get_tokens_clone(&self) -> TokenBackingReference {
        self.tokens.clone()
    }

    fn get_end_position(&self) -> usize {
        self.end_position
    }
}

#[cfg(test)]
#[allow(unused_variables)]
#[allow(unused_imports)]
#[allow(unused_assignments)]
mod tests {
    use crate::cdawg::array_cdawg::ArrayCdawg;
    use crate::cdawg::comparator::CdawgComparator;
    use crate::cdawg::{Cdawg, TopologicalCounter};
    use crate::graph::indexing::{DefaultIx, IndexType, NodeIndex};
    use crate::graph::traits::EdgeRef;
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
        let icdawg = ArrayCdawg::new(cdawg);

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
        let icdawg = ArrayCdawg::new(cdawg);

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
        let icdawg = ArrayCdawg::new(cdawg);

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
        let icdawg = ArrayCdawg::new(cdawg);

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
        let icdawg = ArrayCdawg::new(cdawg);

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
        let icdawg = ArrayCdawg::new(cdawg);

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
    type DiskICdawg = ArrayCdawg<DiskW, DefaultIx, DiskBacking<DiskW, DiskE, DefaultIx>>;

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
        let icdawg = ArrayCdawg::new_mb(
            cdawg,
            icdawg_mb,
            CacheConfig {
                edge_cache_size: 0,
                node_cache_size: 0,
            },
        );
        icdawg.save_metadata(icdawg_path).unwrap();

        let tokens2: Vec<u16> = vec![10, 11, 12];
        let icdawg2: DiskICdawg = ArrayCdawg::load(
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
