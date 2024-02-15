pub mod disk_backing;
pub mod ram_backing;
pub mod vec_backing;

pub use self::disk_backing::DiskBacking;
pub use self::ram_backing::RamBacking;
pub use self::vec_backing::{CacheConfig, CachedDiskVec, DiskVec};

use crate::graph::avl_graph::edge::{Edge, EdgeMutRef, EdgeRef};
use crate::graph::avl_graph::node::{Node, NodeMutRef, NodeRef};

// Define the traits that submodules will implement in various ways.

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

    fn new_node_vec(&self, capacity: Option<usize>, cache_size: usize) -> Self::VecN;

    fn new_edge_vec(&self, capacity: Option<usize>, cache_size: usize) -> Self::VecE;
}

pub trait VecBacking<T> {
    type TRef;
    type TMutRef;

    fn len(&self) -> usize;

    fn push(&mut self, item: T);

    fn index(&self, index: usize) -> Self::TRef;

    fn index_mut(&mut self, index: usize) -> Self::TMutRef;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
