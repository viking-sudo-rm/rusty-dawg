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
}
