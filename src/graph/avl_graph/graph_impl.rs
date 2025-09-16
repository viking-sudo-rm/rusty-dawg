// Graph trait implementation for AvlGraph

use crate::comparator::Comparator;
use crate::graph::avl_graph::AvlGraph;
use crate::graph::graph_trait::Graph;
use crate::graph::indexing::{EdgeIndex, IndexType, NodeIndex};
use crate::memory_backing::MemoryBacking;
use crate::weight::Weight;

// Implement the Graph trait for AvlGraph
impl<N, E, Ix, Mb> Graph<N, E, Ix, Mb::NodeRef, Mb::EdgeRef> for AvlGraph<N, E, Ix, Mb>
where
    Mb: MemoryBacking<N, E, Ix>,
    E: Copy + std::fmt::Debug,
    N: Weight + Copy,
    Ix: IndexType,
    Mb::NodeRef: Copy,
    Mb::EdgeRef: Copy,
{
    fn node_count(&self) -> usize {
        self.node_count()
    }

    fn edge_count(&self) -> usize {
        self.edge_count()
    }

    fn n_edges(&self, node: NodeIndex<Ix>) -> usize {
        self.n_edges(node)
    }

    fn get_node(&self, node: NodeIndex<Ix>) -> Mb::NodeRef {
        self.get_node(node)
    }

    fn get_edge(&self, edge: EdgeIndex<Ix>) -> Mb::EdgeRef {
        self.get_edge(edge)
    }

    fn neighbors(&self, node: NodeIndex<Ix>) -> Box<dyn Iterator<Item = NodeIndex<Ix>> + '_> {
        Box::new(self.neighbors(node))
    }

    fn edges(&self, node: NodeIndex<Ix>) -> Box<dyn Iterator<Item = Mb::EdgeRef> + '_> {
        Box::new(self.edges(node))
    }

    fn get_edge_by_weight_cmp(
        &self,
        node: NodeIndex<Ix>,
        weight: E,
        cmp: Box<dyn Comparator<E>>,
    ) -> Option<EdgeIndex<Ix>> {
        self.get_edge_by_weight_cmp(node, weight, cmp)
    }
}
