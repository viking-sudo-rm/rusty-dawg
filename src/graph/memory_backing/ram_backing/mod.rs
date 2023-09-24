mod vec;

use graph::indexing::IndexType;
use graph::memory_backing::MemoryBacking;
use std::marker::PhantomData;
use weight::Weight;

use graph::avl_graph::edge::Edge;
use graph::avl_graph::node::Node;

pub struct RamBacking<N, E, Ix> {
    marker: PhantomData<(N, E, Ix)>,
}

impl<N, E, Ix> MemoryBacking<N, E, Ix> for RamBacking<N, E, Ix>
where
    Ix: IndexType + Copy,
    N: Weight,
    E: Copy,
{
    type NodeRef = *const Node<N, Ix>;
    type EdgeRef = *const Edge<E, Ix>;
    type NodeMutRef = *mut Node<N, Ix>;
    type EdgeMutRef = *mut Edge<E, Ix>;

    type VecN = Vec<Node<N, Ix>>;
    type VecE = Vec<Edge<E, Ix>>;

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
        RamBacking {
            marker: PhantomData,
        }
    }
}
