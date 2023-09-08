// pub mod byte_field;
pub mod vec_backing;
pub mod node_backing;
pub mod edge_backing;
pub mod ram_backing;

use self::vec_backing::VecBacking;
use self::node_backing::NodeBacking;
use self::edge_backing::EdgeBacking;

pub trait MemoryBacking<N, E, Ix> {
    type Node: NodeBacking<N, Ix>;
    type Edge: EdgeBacking<E, Ix>;

    type VecN: VecBacking<Self::Node>;
    type VecE: VecBacking<Self::Edge>;
}