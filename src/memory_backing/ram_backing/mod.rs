mod vec;

use crate::graph::indexing::IndexType;
use crate::memory_backing::MemoryBacking;
use crate::weight::Weight;
use std::marker::PhantomData;

use crate::graph::avl_graph::edge::Edge;
use crate::graph::avl_graph::node::Node;

#[derive(Clone)]
pub struct RamBacking<N, E, Ix> {
    marker: PhantomData<(N, E, Ix)>,
}

impl<N, E, Ix> MemoryBacking<N, E, Ix> for RamBacking<N, E, Ix>
where
    Ix: IndexType + Copy,
    N: Weight + Clone,
    E: Copy,
{
    type NodeRef = *const Node<N, Ix>;
    type EdgeRef = *const Edge<E, Ix>;
    type NodeMutRef = *mut Node<N, Ix>;
    type EdgeMutRef = *mut Edge<E, Ix>;

    type VecN = Vec<Node<N, Ix>>;
    type VecE = Vec<Edge<E, Ix>>;

    // The disk-backed implementations of new_node_vec and new_edge_vec will presumably pass a file/path.

    fn new_node_vec(&self, capacity: Option<usize>, _cache_size: usize) -> Self::VecN {
        match capacity {
            Some(n) => Vec::with_capacity(n),
            None => Vec::new(),
        }
    }

    fn new_edge_vec(&self, capacity: Option<usize>, _cache_size: usize) -> Self::VecE {
        match capacity {
            Some(n) => Vec::with_capacity(n),
            None => Vec::new(),
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
