pub mod node;
pub mod edge;
mod vec;

use std::marker::PhantomData;
use graph::memory_backing::MemoryBacking;
use graph::indexing::IndexType;

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