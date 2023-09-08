pub mod node;
pub mod edge;
mod vec;

use graph::memory_backing::MemoryBacking;

pub trait RamBacking<N, E, Ix> {}

impl<N, E, Ix> MemoryBacking<N, E, Ix> for dyn RamBacking<N, E, Ix>
where
    Ix: Copy,
{
    type Node = self::node::Node<N, Ix>;
    type Edge = self::edge::Edge<E, Ix>;

    type VecN = Vec<Self::Node>;
    type VecE = Vec<Self::Edge>;
}