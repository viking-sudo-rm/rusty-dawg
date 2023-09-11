pub mod edge;
pub mod node;
// mod vec;       // FIXME: doesn't exist.

// FIXME: Should modify the RamBacking implementation below appropriately.

use graph::indexing::{IndexType, EdgeIndex, NodeIndex};
use graph::memory_backing::MemoryBacking;
use std::marker::PhantomData;

pub struct DiskBacking<N, E, Ix> {
    marker: PhantomData<(N, E, Ix)>,
    nodes_vec: Self::VecN,
    edges_vec: Self::VecE,
}

impl<N, E, Ix> MemoryBacking<N, E, Ix> for DiskBacking<N, E, Ix>
where
    Ix: IndexType + Copy,
{
    type Node = self::node::Node<N, Ix>;
    type Edge = self::edge::Edge<E, Ix>;

    type VecN = DiskVec<Self::Node>;  // FIXME: Need DiskVec here.
    type VecE = DiskVec<Self::Edge>;  // FIXME: Need DiskVec here.

    // The disk-backed implementations of new_node and new_edge will presumably pass a reference to an open file.

    fn new_node(&self, weight: N) -> Self::Node {
        // TODO: should this method just push rather than return?
    }

    fn new_edge(&self, weight: E, target: NodeIndex<Ix>) -> Self::Edge {
        // TODO: should this method just push rather than return an object?
    }

    // The disk-backed implementations of new_node_vec and new_edge_vec will presumably pass a file/path.

    fn new_node_vec(&self, capacity: Option<usize>) -> Self::VecN {
        match capacity {
            Some(n) => Self::VecN::with_capacity(n),
            None => Self::VecN::new(),
        }
    }

    fn new_edge_vec(&self, capacity: Option<usize>) -> Self::VecE {
        match capacity {
            Some(n) => Self::VecE::with_capacity(n),
            None => Self::VecE::new(),
        }
    }
}
