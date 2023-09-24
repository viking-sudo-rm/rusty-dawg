pub mod disk_backing;
pub mod ram_backing;
pub mod vec_backing;

use graph::avl_graph::edge::{Edge, EdgeMutRef, EdgeRef};
use graph::avl_graph::node::{Node, NodeMutRef, NodeRef};

use self::vec_backing::VecBacking;

pub trait MemoryBacking<N, E, Ix> {
    type NodeRef: NodeRef<N, Ix>;
    type EdgeRef: EdgeRef<E, Ix>;
    type NodeMutRef: NodeMutRef<Ix>;
    type EdgeMutRef: EdgeMutRef<Ix>;

    type VecN: VecBacking<Node<N, Ix>, Self::NodeRef, Self::NodeMutRef>;
    type VecE: VecBacking<Edge<E, Ix>, Self::EdgeRef, Self::EdgeMutRef>;

    fn new_node_vec(&self, capacity: Option<usize>) -> Self::VecN;

    fn new_edge_vec(&self, capacity: Option<usize>) -> Self::VecE;
}
