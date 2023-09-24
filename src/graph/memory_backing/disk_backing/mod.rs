// pub mod edge;
// pub mod node;
// pub mod vec;  // Implement VecBacking for DiskVec and DiskVecItem
pub mod disk_vec;

//TODO: Implement all this

use graph::indexing::{EdgeIndex, IndexType, NodeIndex};
use graph::memory_backing::MemoryBacking;
use std::marker::PhantomData;

pub struct DiskBacking<N, E, Ix> {
    marker: PhantomData<(N, E, Ix)>,
    file_path: String,
}

// impl<N, E, Ix> MemoryBacking<N, E, Ix> for DiskBacking<N, E, Ix>
// where
//     Ix: IndexType + Copy,
// {
//     type Node = self::node::Node<N, Ix>;
//     type Edge = self::edge::Edge<E, Ix>;

//     // FIXME: Should these be byte arrays?
//     type VecN = DiskVec<Self::Node>;
//     type VecE = DiskVec<Self::Edge>;

//     // The disk-backed implementations of new_node_vec and new_edge_vec should pass file_path when they construct a new Vector.

//     fn new_node_vec(&self, capacity: Option<usize>) -> Self::VecN {
//         // TODO
//     }

//     fn new_edge_vec(&self, capacity: Option<usize>) -> Self::VecE {
//         // TODO
//     }
// }
