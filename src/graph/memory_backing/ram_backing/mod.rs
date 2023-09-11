pub mod edge;
pub mod node;
mod vec;

use graph::indexing::IndexType;
use graph::memory_backing::MemoryBacking;
use std::marker::PhantomData;

pub struct RamBacking<N, E, Ix> {
    marker: PhantomData<(N, E, Ix)>,
}

impl<N, E, Ix> MemoryBacking<N, E, Ix> for RamBacking<N, E, Ix>
where
    Ix: IndexType + Copy,
{
    type Node = self::node::Node<N, Ix>;
    type Edge = self::edge::Edge<E, Ix>;

    type VecN = Vec<Self::Node>;
    type VecE = Vec<Self::Edge>;

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

impl<N, E, Ix> Default for RamBacking<N, E, Ix>
where
    Ix: IndexType + Copy,
{
    fn default() -> Self {
        RamBacking {marker: PhantomData}
    }
}