// Common trait for graph implementations (both AvlGraph and ArrayGraph)

use crate::comparator::Comparator;
use crate::graph::indexing::{EdgeIndex, IndexType, NodeIndex};
use crate::graph::traits::{EdgeRef, NodeRef};
use crate::weight::Weight;

/// Common trait for graph implementations
pub trait Graph<N, E, Ix, Node, Edge>
where
    Ix: IndexType,
    N: Weight,
    E: Copy + std::fmt::Debug,
    Node: NodeRef<N, Ix> + Copy,
    Edge: EdgeRef<E, Ix> + Copy,
{
    // Associated types for node and edge references

    // Basic graph information
    fn node_count(&self) -> usize;
    fn edge_count(&self) -> usize;
    fn n_edges(&self, node: NodeIndex<Ix>) -> usize;

    // Node and edge access
    fn get_node(&self, node: NodeIndex<Ix>) -> Node;
    fn get_edge(&self, edge: EdgeIndex<Ix>) -> Edge;

    // Graph traversal
    fn neighbors(&self, node: NodeIndex<Ix>) -> Box<dyn Iterator<Item = NodeIndex<Ix>> + '_>;
    fn edges(&self, node: NodeIndex<Ix>) -> Box<dyn Iterator<Item = Edge> + '_>;

    // Edge finding
    fn get_edge_by_weight_cmp(
        &self,
        node: NodeIndex<Ix>,
        weight: E,
        cmp: Box<dyn Comparator<E>>,
    ) -> Option<EdgeIndex<Ix>>;
}

/// Trait for node references that provides common functionality
pub trait GraphNodeRef<N, Ix>
where
    Ix: IndexType,
    N: Weight,
{
    fn get_weight(self) -> N
    where
        N: Clone;
    fn get_length(self) -> u64;
    fn get_failure(self) -> Option<NodeIndex<Ix>>;
    fn get_count(self) -> usize;
}

/// Trait for edge references that provides common functionality
pub trait GraphEdgeRef<E, Ix>
where
    Ix: IndexType,
    E: Copy,
{
    fn get_weight(self) -> E;
    fn get_target(self) -> NodeIndex<Ix>;
}
