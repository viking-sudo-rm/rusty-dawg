// pub mod disk_backing;
pub mod ram_backing;
pub mod vec_backing;

use graph::avl_graph::node::{Node, MutNode};
use graph::avl_graph::edge::{Edge, MutEdge};

use self::vec_backing::VecBacking;

pub trait MemoryBacking<N, E, Ix> {
    type MutNode: MutNode<Ix>;
    type MutEdge: MutEdge<Ix>;

    type VecN: VecBacking<Node<N, Ix>, Self::MutNode>;
    type VecE: VecBacking<Edge<E, Ix>, Self::MutEdge>;

    fn new_node_vec(&self, capacity: Option<usize>) -> Self::VecN;

    fn new_edge_vec(&self, capacity: Option<usize>) -> Self::VecE;
}
