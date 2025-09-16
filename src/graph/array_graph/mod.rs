// An immutable array-based graph

use crate::comparator::Comparator;
use anyhow::Result;
use std::clone::Clone;
use std::cmp::Ordering;
use std::path::Path;

use crate::graph::avl_graph::AvlGraph;
use crate::graph::indexing::{DefaultIx, EdgeIndex, IndexType, NodeIndex};
use crate::graph::traits::{EdgeRef, NodeRef};
use crate::memory_backing::{
    ArrayMemoryBacking, CacheConfig, DiskVec, InternallyImmutableVecBacking, MemoryBacking,
};
use crate::serde::de::DeserializeOwned;
use crate::serde::Serialize;
use crate::weight::Weight;
use std::fmt::Debug;

pub mod edge;
mod graph_impl;
pub mod node;
mod serde;

pub use self::edge::ArrayEdge;
pub use self::node::{ArrayNode, ArrayNodeRef};

use crate::memory_backing::RamBacking;
use crate::memory_backing::{disk_backing, DiskBacking};

#[derive(Default)]
pub struct ArrayGraph<N, E, Ix = DefaultIx, Mb = RamBacking<N, E, Ix>>
where
    Mb: ArrayMemoryBacking<N, E, Ix>,
    Ix: IndexType,
{
    nodes: Mb::ArrayVecN,
    edges: Mb::ArrayVecE,
}

impl<N, E, Ix> ArrayGraph<N, E, Ix>
where
    E: Copy + Debug,
    Ix: IndexType,
    N: Weight + Clone,
{
    pub fn new(mutable_graph: AvlGraph<N, E, Ix>) -> Self {
        let mb: RamBacking<N, E, Ix> = RamBacking::default();
        Self::new_mb(
            mutable_graph,
            mb,
            CacheConfig {
                node_cache_size: 0,
                edge_cache_size: 0,
            },
        )
    }

    pub fn save_to_disk<P: AsRef<Path> + Clone + Debug>(&self, path: P) -> Result<()>
    where
        N: Serialize + DeserializeOwned + Default,
        E: Serialize + DeserializeOwned + Default,
        Ix: Serialize + DeserializeOwned + Default,
    {
        let mb: DiskBacking<N, E, Ix> = DiskBacking::new(path);
        let _ = DiskVec::from_vec(&self.nodes, mb.get_nodes_path());
        let _ = DiskVec::from_vec(&self.edges, mb.get_edges_path());
        Ok(())
    }
}

impl<N, E, Ix> ArrayGraph<N, E, Ix, DiskBacking<N, E, Ix>>
where
    E: Copy + Debug + Serialize + DeserializeOwned + Default,
    N: Weight + Copy + Clone + Serialize + DeserializeOwned + Default,
    Ix: IndexType + Serialize + DeserializeOwned,
{
    pub fn load<P: AsRef<Path> + Clone + Debug>(
        path: P,
        cache_config: CacheConfig,
    ) -> Result<Self> {
        let mb: DiskBacking<N, E, Ix> = DiskBacking::new(path);
        // FIXME: This can be refactored to call a method in Mb.
        let nodes =
            disk_backing::vec::Vec::load(mb.get_nodes_path(), cache_config.node_cache_size)?;
        let edges =
            disk_backing::vec::Vec::load(mb.get_edges_path(), cache_config.edge_cache_size)?;
        Ok(Self { nodes, edges })
    }
}

impl<N, E, Ix, Mb> ArrayGraph<N, E, Ix, Mb>
where
    N: Weight + Clone,
    Mb: ArrayMemoryBacking<N, E, Ix>,
    E: Copy + Debug,
    Ix: IndexType,
{
    pub fn new_mb<SourceMb: MemoryBacking<N, E, Ix>>(
        mutable_graph: AvlGraph<N, E, Ix, SourceMb>,
        mb: Mb,
        cache_config: CacheConfig,
    ) -> Self {
        /* TODO make an online way to do this that does not require double the RAM if copying from
        ram AVLGraph to a ram array graph. */

        let mut nodes = mb.new_array_node_vec(
            Some(mutable_graph.node_count()),
            cache_config.node_cache_size,
        );
        let mut edges = mb.new_array_edge_vec(
            Some(mutable_graph.edge_count()),
            cache_config.edge_cache_size,
        );
        /* Maybe these should be Ix types, but my hunch is the arithmetic will be faster with usize
         * and they're not being stored as usize
         */
        let mut edge_index: usize = 0;
        let mut node_index: usize = 0;

        while node_index < mutable_graph.node_count() {
            // default values
            let mut first_edge: EdgeIndex<Ix> = EdgeIndex::end();
            let mut num_edges = 0;

            if mutable_graph
                .get_node(NodeIndex::new(node_index))
                .get_first_edge()
                != EdgeIndex::end()
            {
                first_edge = EdgeIndex::new(edge_index);
                for avl_edge in mutable_graph.ordered_edges(NodeIndex::new(node_index)) {
                    num_edges += 1;
                    edges.push(ArrayEdge {
                        weight: avl_edge.get_weight(),
                        target: avl_edge.get_target(),
                    });
                    edge_index += 1;
                }
            }
            nodes.push(ArrayNode {
                weight: mutable_graph
                    .get_node(NodeIndex::new(node_index))
                    .get_weight(),
                first_edge,
                num_edges,
            });
            node_index += 1;
        }
        // TODO: Make sure the AVL Graph is getting freed here. Maybe implement the Drop trait.
        ArrayGraph { nodes, edges }
    }
}

impl<N, E, Ix, Mb> ArrayGraph<N, E, Ix, Mb>
where
    Mb: ArrayMemoryBacking<N, E, Ix>,
    E: Copy + Debug,
    N: Weight,
    Ix: IndexType,
{
    // Given a node, find if it has an edge of the specified weight
    pub fn get_edge_by_weight_cmp(
        &self,
        a: NodeIndex<Ix>,
        weight: E,
        cmp: Box<dyn Comparator<E>>,
    ) -> Option<EdgeIndex<Ix>> {
        let num_edges = self.get_node(a).get_num_edges().index();
        if num_edges == 0 {
            return None;
        }
        let first_edge = self.get_node(a).get_first_edge().index();
        self.binary_search(first_edge.index(), first_edge + num_edges, weight, cmp)
            .map(EdgeIndex::new)
    }

    /**
     * Internal helper to find an edge
     *
     * start: first edge in the search range (inclusive)
     * stop: last edge in the search range (exclusive)
     * target_weight: the weight of the edge to find
     * cmp: comparator to use.
     */
    fn binary_search(
        &self,
        start: usize,
        stop: usize,
        target_weight: E,
        cmp: Box<dyn Comparator<E>>,
    ) -> Option<usize> {
        if start == stop {
            return None;
        }
        let mid = (start + stop) / 2;
        let mid_weight = self.edges.index(mid).get_weight();
        match cmp.compare(&target_weight, &mid_weight) {
            Ordering::Equal => Some(mid),
            Ordering::Less => self.binary_search(start, mid, target_weight, cmp),
            Ordering::Greater => self.binary_search(mid + 1, stop, target_weight, cmp),
        }
    }

    pub fn n_edges(&self, a: NodeIndex<Ix>) -> usize {
        self.nodes.index(a.index()).get_num_edges().index()
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    pub fn neighbors(&self, node: NodeIndex<Ix>) -> Neighbors<'_, N, E, Ix, Mb> {
        Neighbors::new(self, node)
    }

    pub fn edges(&self, edges: NodeIndex<Ix>) -> Edges<'_, N, E, Ix, Mb> {
        Edges::new(self, edges)
    }

    // We can't use standard indexing because we have custom reference types.

    pub fn get_node(&self, node: NodeIndex<Ix>) -> Mb::ArrayNodeRef {
        self.nodes.index(node.index())
    }

    pub fn get_edge(&self, edge: EdgeIndex<Ix>) -> Mb::ArrayEdgeRef {
        self.edges.index(edge.index())
    }
}

pub struct Neighbors<'a, N, E, Ix, Mb>
where
    Mb: ArrayMemoryBacking<N, E, Ix>,
    Ix: IndexType,
{
    edges: Edges<'a, N, E, Ix, Mb>,
}

impl<N, E, Ix, Mb> Iterator for Neighbors<'_, N, E, Ix, Mb>
where
    Mb: ArrayMemoryBacking<N, E, Ix>,
    Ix: IndexType,
{
    type Item = NodeIndex<Ix>;

    fn next(&mut self) -> Option<NodeIndex<Ix>> {
        self.edges.next().map(|edge| edge.get_target())
    }
}

impl<'a, N, E, Ix, Mb> Neighbors<'a, N, E, Ix, Mb>
where
    Mb: ArrayMemoryBacking<N, E, Ix>,
    Ix: IndexType,
{
    pub fn new(graph: &'a ArrayGraph<N, E, Ix, Mb>, node: NodeIndex<Ix>) -> Self {
        let edges = Edges::new(graph, node);
        Self { edges }
    }
}

pub struct Edges<'a, N, E, Ix, Mb>
where
    Mb: ArrayMemoryBacking<N, E, Ix>,
    Ix: IndexType,
{
    graph: &'a ArrayGraph<N, E, Ix, Mb>,
    // the next edge to return -- keeping this as usize since we won't have a ton of them
    index: usize,
    // the end of the range to traverse -- exclusive
    end: usize,
}

impl<N, E, Ix, Mb> Iterator for Edges<'_, N, E, Ix, Mb>
where
    Mb: ArrayMemoryBacking<N, E, Ix>,
    Ix: IndexType,
    Mb::ArrayEdgeRef: Sized,
{
    type Item = Mb::ArrayEdgeRef;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.end {
            None
        } else {
            let to_return = self.index.index();
            self.index += 1;
            Some(self.graph.edges.index(to_return))
        }
    }
}

impl<'a, N, E, Ix, Mb> Edges<'a, N, E, Ix, Mb>
where
    Mb: ArrayMemoryBacking<N, E, Ix>,
    Ix: IndexType,
{
    pub fn new(graph: &'a ArrayGraph<N, E, Ix, Mb>, node: NodeIndex<Ix>) -> Self {
        let index = graph.nodes.index(node.index()).get_first_edge().index();
        let end = index + graph.nodes.index(node.index()).get_num_edges().index();
        Self { graph, index, end }
    }
}

#[cfg(test)]
#[allow(unused_variables)]
#[allow(unused_imports)]
mod tests {
    use crate::graph::array_graph::ArrayGraph;
    use crate::graph::avl_graph::AvlGraph;
    use crate::graph::comparator::DEFAULT_CMP;
    use crate::graph::indexing::{EdgeIndex, NodeIndex};
    use crate::graph::traits::EdgeRef;
    use crate::weight::{DefaultWeight, Weight};

    fn generate_avl_graph() -> AvlGraph<DefaultWeight, u16> {
        let weight = DefaultWeight::new(0, None, 0);
        let weight1 = DefaultWeight::new(1, None, 1);
        let mut avl_graph: AvlGraph<DefaultWeight, u16> = AvlGraph::new();
        let q0 = avl_graph.add_node(weight);
        let q1 = avl_graph.add_node(weight1);

        avl_graph.add_balanced_edge(q0, q1, 4);
        avl_graph.add_balanced_edge(q0, q1, 1);
        avl_graph.add_balanced_edge(q0, q1, 0);
        avl_graph.add_balanced_edge(q0, q1, 3);
        avl_graph.add_balanced_edge(q0, q1, 2);

        avl_graph
    }
    #[test]
    fn test_create_graph() {
        let graph = ArrayGraph::new(generate_avl_graph());

        assert_eq!(graph.nodes.len(), 2);
        let target;
        let source;

        // We don't impose any ordering on the nodes.
        if graph.nodes[0].weight.get_count() == 0 {
            target = 1;
            source = 0;
        } else {
            target = 0;
            source = 1;
        }

        assert_eq!(graph.nodes[source].weight.get_length(), 0);
        assert_eq!(graph.nodes[source].weight.get_failure(), None);
        assert_eq!(graph.nodes[source].first_edge.index(), 0);
        assert_eq!(graph.nodes[source].num_edges, 5);

        assert_eq!(graph.nodes[target].weight.get_length(), 1);
        assert_eq!(graph.nodes[target].weight.get_failure(), None);
        assert_eq!(graph.nodes[target].weight.get_count(), 1);
        assert_eq!(graph.nodes[target].first_edge, EdgeIndex::end());
        assert_eq!(graph.nodes[target].num_edges, 0);

        for i in 0..5 {
            assert_eq!(graph.edges[i].weight, i as u16);
            assert_eq!(graph.edges[i].target.index(), target);
        }
    }

    #[test]
    fn test_get_edge_by_weight_cmp() {
        let graph = ArrayGraph::new(generate_avl_graph());
        let source_node = NodeIndex::new(if graph.nodes[0].num_edges != 0 { 0 } else { 1 });

        for i in 0..5 {
            assert_eq!(
                graph.get_edge_by_weight_cmp(source_node, i, Box::new(DEFAULT_CMP)),
                Some(EdgeIndex::new(i as usize))
            );
        }

        assert_eq!(
            graph.get_edge_by_weight_cmp(source_node, 6, Box::new(DEFAULT_CMP)),
            None
        );
    }

    #[test]
    fn test_edges() {
        let graph = ArrayGraph::new(generate_avl_graph());
        let source_node = NodeIndex::new(if graph.nodes[0].num_edges != 0 { 0 } else { 1 });

        for (i, edge) in graph.edges(source_node).enumerate() {
            assert_eq!(edge.get_weight(), i as u16);
        }
    }
}
