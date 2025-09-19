use crate::graph::avl_graph::edge::{AvlEdge, AvlEdgeMutRef};
use crate::graph::avl_graph::node::{AvlNode, AvlNodeMutRef};
use crate::memory_backing::disk_backing::{EdgeIndex, IndexType, NodeIndex};
use crate::memory_backing::CachedDiskVec;
use crate::weight::Weight;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::cell::RefCell;
use std::rc::Rc;

pub trait MutRef<T> {
    fn new(disk_vec: Rc<RefCell<CachedDiskVec<T>>>, index: usize) -> Self;
}

pub struct DiskNodeMutRef<N, Ix> {
    disk_vec: Rc<RefCell<CachedDiskVec<AvlNode<N, Ix>>>>,
    index: usize,
}

impl<N, Ix> MutRef<AvlNode<N, Ix>> for DiskNodeMutRef<N, Ix> {
    fn new(disk_vec: Rc<RefCell<CachedDiskVec<AvlNode<N, Ix>>>>, index: usize) -> Self {
        Self { disk_vec, index }
    }
}

// TODO: Only overwrite the specific field in the DiskVec rather than read/write.
impl<N, Ix> AvlNodeMutRef<Ix> for DiskNodeMutRef<N, Ix>
where
    Ix: IndexType,
    N: Weight,
    AvlNode<N, Ix>: Serialize + DeserializeOwned + Default + Copy,
{
    fn set_length(self, length: u64) {
        let mut disk_vec = self.disk_vec.borrow_mut();
        let mut node = disk_vec.get(self.index).unwrap();
        node.weight.set_length(length);
        let _ = disk_vec.set(self.index, &node);
    }

    fn set_failure(self, state: Option<NodeIndex<Ix>>) {
        let mut disk_vec = self.disk_vec.borrow_mut();
        let mut node = disk_vec.get(self.index).unwrap();
        // Handle potential mismatch in Ix.
        let fail_state = state.map(|phi| NodeIndex::new(phi.index()));
        node.weight.set_failure(fail_state);
        let _ = disk_vec.set(self.index, &node);
    }

    fn increment_count(self) {
        let mut disk_vec = self.disk_vec.borrow_mut();
        let mut node = disk_vec.get(self.index).unwrap();
        node.weight.increment_count();
        let _ = disk_vec.set(self.index, &node);
    }

    fn set_count(self, count: usize) {
        let mut disk_vec = self.disk_vec.borrow_mut();
        let mut node = disk_vec.get(self.index).unwrap();
        node.weight.set_count(count);
        let _ = disk_vec.set(self.index, &node);
    }

    fn set_first_edge(self, first_edge: EdgeIndex<Ix>) {
        let mut disk_vec = self.disk_vec.borrow_mut();
        let mut node = disk_vec.get(self.index).unwrap();
        node.first_edge = first_edge;
        let _ = disk_vec.set(self.index, &node);
    }
}

pub struct DiskEdgeMutRef<E, Ix> {
    disk_vec: Rc<RefCell<CachedDiskVec<AvlEdge<E, Ix>>>>,
    index: usize,
}

impl<E, Ix> MutRef<AvlEdge<E, Ix>> for DiskEdgeMutRef<E, Ix> {
    fn new(disk_vec: Rc<RefCell<CachedDiskVec<AvlEdge<E, Ix>>>>, index: usize) -> Self {
        Self { disk_vec, index }
    }
}

impl<E, Ix> AvlEdgeMutRef<E, Ix> for DiskEdgeMutRef<E, Ix>
where
    Ix: IndexType + Copy,
    AvlEdge<E, Ix>: Serialize + DeserializeOwned + Default,
    E: Copy,
{
    fn set_weight(self, weight: E) {
        let mut disk_vec = self.disk_vec.borrow_mut();
        let mut edge = disk_vec.get(self.index).unwrap();
        edge.weight = weight;
        let _ = disk_vec.set(self.index, &edge);
    }

    fn set_target(self, target: NodeIndex<Ix>) {
        let mut disk_vec = self.disk_vec.borrow_mut();
        let mut edge = disk_vec.get(self.index).unwrap();
        edge.target = target;
        let _ = disk_vec.set(self.index, &edge);
    }

    fn set_left(self, left: EdgeIndex<Ix>) {
        let mut disk_vec = self.disk_vec.borrow_mut();
        let mut edge = disk_vec.get(self.index).unwrap();
        edge.left = left;
        let _ = disk_vec.set(self.index, &edge);
    }

    fn set_right(self, right: EdgeIndex<Ix>) {
        let mut disk_vec = self.disk_vec.borrow_mut();
        let mut edge = disk_vec.get(self.index).unwrap();
        edge.right = right;
        let _ = disk_vec.set(self.index, &edge);
    }

    fn set_balance_factor(self, bf: i8) {
        let mut disk_vec = self.disk_vec.borrow_mut();
        let mut edge = disk_vec.get(self.index).unwrap();
        edge.balance_factor = bf;
        let _ = disk_vec.set(self.index, &edge);
    }
}

pub trait DiskVecItem: Sized {
    type MutRef: MutRef<Self>;
}

impl<N, Ix> DiskVecItem for AvlNode<N, Ix> {
    type MutRef = DiskNodeMutRef<N, Ix>;
}

impl<E, Ix> DiskVecItem for AvlEdge<E, Ix> {
    type MutRef = DiskEdgeMutRef<E, Ix>;
}
