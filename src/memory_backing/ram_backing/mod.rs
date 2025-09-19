mod vec;

use crate::graph::indexing::IndexType;
use crate::memory_backing::{ArrayMemoryBacking, MemoryBacking};
use crate::weight::Weight;
use std::marker::PhantomData;

use crate::graph::array_graph::edge::ArrayEdge;
use crate::graph::array_graph::node::ArrayNode;
use crate::graph::avl_graph::edge::AvlEdge;
use crate::graph::avl_graph::node::AvlNode;

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
    type NodeRef = *const AvlNode<N, Ix>;
    type EdgeRef = *const AvlEdge<E, Ix>;
    type NodeMutRef = *mut AvlNode<N, Ix>;
    type EdgeMutRef = *mut AvlEdge<E, Ix>;

    type VecN = Vec<AvlNode<N, Ix>>;
    type VecE = Vec<AvlEdge<E, Ix>>;

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

impl<N, E, Ix> ArrayMemoryBacking<N, E, Ix> for RamBacking<N, E, Ix>
where
    Ix: IndexType + Copy,
    N: Weight + Clone,
    E: Copy,
{
    type ArrayNodeRef = *const ArrayNode<N, Ix>;
    type ArrayEdgeRef = *const ArrayEdge<E, Ix>;

    type ArrayVecN = Vec<ArrayNode<N, Ix>>;
    type ArrayVecE = Vec<ArrayEdge<E, Ix>>;

    // The disk-backed implementations of new_node_vec and new_edge_vec will presumably pass a file/path.
    // Could probably remove some repeated code here -- but I don't want to leap in premature abstraction
    fn new_array_node_vec(&self, capacity: Option<usize>, _cache_size: usize) -> Self::ArrayVecN {
        match capacity {
            Some(n) => Vec::with_capacity(n),
            None => Vec::new(),
        }
    }

    fn new_array_edge_vec(&self, capacity: Option<usize>, _cache_size: usize) -> Self::ArrayVecE {
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
