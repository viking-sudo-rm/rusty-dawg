// Graph trait implementation for ArrayGraph

use crate::comparator::Comparator;
use crate::graph::array_graph::ArrayGraph;
use crate::graph::graph_trait::Graph;
use crate::graph::indexing::{EdgeIndex, IndexType, NodeIndex};
use crate::memory_backing::ArrayMemoryBacking;
use crate::weight::Weight;

// Implement the Graph trait for ArrayGraph
impl<N, E, Ix, Mb> Graph<N, E, Ix, Mb::ArrayNodeRef, Mb::ArrayEdgeRef> for ArrayGraph<N, E, Ix, Mb>
where
    Mb: ArrayMemoryBacking<N, E, Ix>,
    E: Copy + std::fmt::Debug,
    N: Weight,
    Ix: IndexType,
    Mb::ArrayNodeRef: Copy,
    Mb::ArrayEdgeRef: Copy,
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

    fn get_node(&self, node: NodeIndex<Ix>) -> Mb::ArrayNodeRef {
        self.get_node(node)
    }

    fn get_edge(&self, edge: EdgeIndex<Ix>) -> Mb::ArrayEdgeRef {
        self.get_edge(edge)
    }

    fn neighbors(&self, node: NodeIndex<Ix>) -> Box<dyn Iterator<Item = NodeIndex<Ix>> + '_> {
        Box::new(self.neighbors(node))
    }

    fn edges(&self, node: NodeIndex<Ix>) -> Box<dyn Iterator<Item = Mb::ArrayEdgeRef> + '_> {
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
