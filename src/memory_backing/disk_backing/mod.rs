mod disk_mut_refs;
pub mod vec; // Implement VecBacking for DiskVec and DiskVecItem // Raw implementation of DiskVec data structure.

use crate::graph::array_graph::edge::ArrayEdge;
use crate::graph::array_graph::node::ArrayNode;
use crate::graph::avl_graph::edge::AvlEdge;
use crate::graph::avl_graph::node::AvlNode;

use crate::graph::indexing::{EdgeIndex, IndexType, NodeIndex};
use crate::memory_backing::{ArrayMemoryBacking, MemoryBacking};
use crate::weight::Weight;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fs::create_dir_all;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

use self::disk_mut_refs::{DiskEdgeMutRef, DiskNodeMutRef};
use self::vec::Vec;

#[derive(Clone)]
pub struct DiskBacking<N, E, Ix> {
    dir_path: Box<Path>,
    marker: PhantomData<(N, E, Ix)>,
}

impl<N, E, Ix> DiskBacking<N, E, Ix> {
    pub fn new<P: AsRef<Path> + Clone + std::fmt::Debug>(dir_path: P) -> Self {
        create_dir_all(dir_path.clone()).unwrap();
        Self {
            dir_path: Box::from(dir_path.as_ref()),
            marker: PhantomData,
        }
    }

    pub fn get_nodes_path(&self) -> PathBuf {
        self.dir_path.join("nodes.vec")
    }

    pub fn get_edges_path(&self) -> PathBuf {
        self.dir_path.join("edges.vec")
    }
}

impl<N, E, Ix> MemoryBacking<N, E, Ix> for DiskBacking<N, E, Ix>
where
    Ix: IndexType + Copy + Serialize + DeserializeOwned,
    N: Weight + Serialize + DeserializeOwned + Default + Clone + Copy,
    E: Copy + Serialize + DeserializeOwned + Default + Copy,
{
    type NodeRef = AvlNode<N, Ix>;
    type EdgeRef = AvlEdge<E, Ix>;
    type NodeMutRef = DiskNodeMutRef<N, Ix>;
    type EdgeMutRef = DiskEdgeMutRef<E, Ix>;

    // This Vec type wraps a DiskVec in an Rc<RefCell<..>>
    type VecN = Vec<AvlNode<N, Ix>>;
    type VecE = Vec<AvlEdge<E, Ix>>;

    // The disk-backed implementations of new_node_vec and new_edge_vec should pass file_path when they construct a new Vector.

    fn new_node_vec(&self, capacity: Option<usize>, cache_size: usize) -> Self::VecN {
        let path = self.get_nodes_path();
        match capacity {
            Some(n) => Vec::new(path, n, cache_size).unwrap(),
            None => Vec::new(path, 8, cache_size).unwrap(),
        }
    }

    fn new_edge_vec(&self, capacity: Option<usize>, cache_size: usize) -> Self::VecE {
        let path = self.get_edges_path();
        match capacity {
            Some(n) => Vec::new(path, n, cache_size).unwrap(),
            None => Vec::new(path, 8, cache_size).unwrap(),
        }
    }
}

impl<N, E, Ix> ArrayMemoryBacking<N, E, Ix> for DiskBacking<N, E, Ix>
where
    Ix: IndexType + Copy + Serialize + DeserializeOwned,
    N: Weight + Serialize + DeserializeOwned + Default + Clone + Copy,
    E: Copy + Serialize + DeserializeOwned + Default + Copy,
{
    type ArrayNodeRef = ArrayNode<N, Ix>;
    type ArrayEdgeRef = ArrayEdge<E, Ix>;

    // This Vec type wraps a DiskVec in an Rc<RefCell<..>>
    type ArrayVecN = Vec<ArrayNode<N, Ix>>;
    type ArrayVecE = Vec<ArrayEdge<E, Ix>>;

    // The disk-backed implementations of new_node_vec and new_edge_vec should pass file_path when they construct a new Vector.
    // Could probably remove some repeated code here -- but I don't want to leap in premature abstraction
    fn new_array_node_vec(&self, capacity: Option<usize>, cache_size: usize) -> Self::ArrayVecN {
        let path = self.get_nodes_path();
        match capacity {
            Some(n) => Vec::new(path, n, cache_size).unwrap(),
            None => Vec::new(path, 8, cache_size).unwrap(),
        }
    }

    fn new_array_edge_vec(&self, capacity: Option<usize>, cache_size: usize) -> Self::ArrayVecE {
        let path = self.get_edges_path();
        match capacity {
            Some(n) => Vec::new(path, n, cache_size).unwrap(),
            None => Vec::new(path, 8, cache_size).unwrap(),
        }
    }
}
