pub mod disk_backing;
pub mod ram_backing;
pub mod vec_backing;

pub use self::disk_backing::DiskBacking;
pub use self::ram_backing::RamBacking;
pub use self::vec_backing::{CacheConfig, CachedDiskVec, DiskVec};
use crate::graph::array_graph::{ArrayEdge, ArrayNode};

use crate::graph::array_graph::node::ArrayNodeRef;
use crate::graph::avl_graph::edge::{AvlEdge, AvlEdgeMutRef, AvlEdgeRef};
use crate::graph::avl_graph::node::{AvlNode, AvlNodeMutRef};
use crate::graph::traits::{EdgeRef, NodeRef};
// Define the traits that submodules will implement in various ways.

pub trait MemoryBacking<N, E, Ix>
where
    Self: Clone,
    Self::NodeRef: Copy,
    Self::EdgeRef: Copy,
{
    type NodeRef: NodeRef<N, Ix>;
    type EdgeRef: AvlEdgeRef<E, Ix>;
    type NodeMutRef: AvlNodeMutRef<Ix>;
    type EdgeMutRef: AvlEdgeMutRef<E, Ix>;

    type VecN: VecBacking<AvlNode<N, Ix>, TRef = Self::NodeRef, TMutRef = Self::NodeMutRef>;
    type VecE: VecBacking<AvlEdge<E, Ix>, TRef = Self::EdgeRef, TMutRef = Self::EdgeMutRef>;

    fn new_node_vec(&self, capacity: Option<usize>, cache_size: usize) -> Self::VecN;

    fn new_edge_vec(&self, capacity: Option<usize>, cache_size: usize) -> Self::VecE;
}

pub trait InternallyImmutableVecBacking<T> {
    type TRef;

    fn len(&self) -> usize;

    fn index(&self, index: usize) -> Self::TRef;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn set(&mut self, index: usize, value: T);

    fn push(&mut self, item: T);
}

pub trait VecBacking<T>: InternallyImmutableVecBacking<T> {
    type TMutRef;

    fn index_mut(&mut self, index: usize) -> Self::TMutRef;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub trait ArrayMemoryBacking<N, E, Ix>
where
    Self: Clone,
    Self::ArrayEdgeRef: Copy,
{
    type ArrayNodeRef: ArrayNodeRef<N, Ix>;
    type ArrayEdgeRef: EdgeRef<E, Ix>;

    type ArrayVecN: InternallyImmutableVecBacking<ArrayNode<N, Ix>, TRef = Self::ArrayNodeRef>;
    type ArrayVecE: InternallyImmutableVecBacking<ArrayEdge<E, Ix>, TRef = Self::ArrayEdgeRef>;

    fn new_array_node_vec(&self, capacity: Option<usize>, cache_size: usize) -> Self::ArrayVecN;

    fn new_array_edge_vec(&self, capacity: Option<usize>, cache_size: usize) -> Self::ArrayVecE;
}
