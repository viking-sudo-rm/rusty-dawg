// pub mod edge;  // FIXME: has pseudo-code for DiskVec API.
// pub mod node;  // FIXME: has pseudo-code for DiskVec API.
// mod vec;       // FIXME: doesn't exist.

// FIXME: Should modify the RamBacking implementation below appropriately.

// use graph::indexing::{IndexType, EdgeIndex, NodeIndex};
// use graph::memory_backing::MemoryBacking;
// use std::marker::PhantomData;

// pub struct RamBacking<N, E, Ix> {
//     marker: PhantomData<(N, E, Ix)>,
// }

// impl<N, E, Ix> MemoryBacking<N, E, Ix> for RamBacking<N, E, Ix>
// where
//     Ix: IndexType + Copy,
// {
//     type Node = self::node::Node<N, Ix>;
//     type Edge = self::edge::Edge<E, Ix>;

//     type VecN = Vec<Self::Node>;
//     type VecE = Vec<Self::Edge>;

//     // The disk-backed implementations of new_node and new_edge will presumably pass a reference to an open file.

//     fn new_node(&self, weight: N) -> Self::Node {
//         Self::Node {
//             weight,
//             first_edge: EdgeIndex::end(),
//         }
//     }

//     fn new_edge(&self, weight: E, target: NodeIndex<Ix>) -> Self::Edge {
//         Self::Edge {
//             weight,
//             target,
//             left: EdgeIndex::end(),
//             right: EdgeIndex::end(),
//             balance_factor: 0,
//         }
//     }

//     // The disk-backed implementations of new_node_vec and new_edge_vec will presumably pass a file/path.

//     fn new_node_vec(&self, capacity: Option<usize>) -> Self::VecN {
//         match capacity {
//             Some(n) => Self::VecN::with_capacity(n),
//             None => Self::VecN::new(),
//         }
//     }

//     fn new_edge_vec(&self, capacity: Option<usize>) -> Self::VecE {
//         match capacity {
//             Some(n) => Self::VecE::with_capacity(n),
//             None => Self::VecE::new(),
//         }
//     }
// }
