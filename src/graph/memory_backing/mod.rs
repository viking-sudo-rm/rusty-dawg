pub mod disk_backing;
pub mod ram_backing;
pub mod vec_backing;

pub use self::disk_backing::DiskBacking;
pub use self::ram_backing::RamBacking;

use crate::graph::avl_graph::edge::{Edge, EdgeMutRef, EdgeRef};
use crate::graph::avl_graph::node::{Node, NodeMutRef, NodeRef};

use self::vec_backing::VecBacking;

pub trait MemoryBacking<N, E, Ix>
where
    Self: Clone,
    Self::EdgeRef: Copy,
{
    type NodeRef: NodeRef<N, Ix>;
    type EdgeRef: EdgeRef<E, Ix>;
    type NodeMutRef: NodeMutRef<Ix>;
    type EdgeMutRef: EdgeMutRef<E, Ix>;

    type VecN: VecBacking<Node<N, Ix>, TRef = Self::NodeRef, TMutRef = Self::NodeMutRef>;
    type VecE: VecBacking<Edge<E, Ix>, TRef = Self::EdgeRef, TMutRef = Self::EdgeMutRef>;

    fn new_node_vec(&self, capacity: Option<usize>) -> Self::VecN;

    fn new_edge_vec(&self, capacity: Option<usize>) -> Self::VecE;
}
