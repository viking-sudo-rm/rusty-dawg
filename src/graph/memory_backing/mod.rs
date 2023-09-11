// pub mod byte_field;
pub mod edge_backing;
pub mod node_backing;
pub mod ram_backing;
pub mod vec_backing;

use self::edge_backing::EdgeBacking;
use self::node_backing::NodeBacking;
use self::vec_backing::VecBacking;

pub trait MemoryBacking<N, E, Ix> {
    type Node: NodeBacking<N, Ix>;
    type Edge: EdgeBacking<E, Ix>;

    type VecN: VecBacking<Self::Node>;
    type VecE: VecBacking<Self::Edge>;

    fn new_node_vec(&self, capacity: Option<usize>) -> Self::VecN;

    fn new_edge_vec(&self, capacity: Option<usize>) -> Self::VecE;
}
