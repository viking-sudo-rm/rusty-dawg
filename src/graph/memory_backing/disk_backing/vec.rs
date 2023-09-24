// Implement the VecBacking interface for DiskVec.

use graph::memory_backing::vec_backing::VecBacking;
use graph::avl_graph::node::Node;
use graph::avl_graph::edge::Edge;
use graph::memory_backing::disk_backing::disk_mut_refs::{DiskNodeMutRef, DiskEdgeMutRef};

pub trait DiskVecItem {
    type Ref;
    type MutRef;
}

impl<N, Ix> DiskVecItem for Node<N, Ix> {
    type Ref = Node<N, Ix>;
    type MutRef = DiskNodeMutRef<N, Ix>;
}

impl<E, Ix> DiskVecItem for Edge<E, Ix> {
    type Ref = Edge<E, Ix>;
    type MutRef = DiskEdgeMutRef<E, Ix>;
}

impl<T> VecBacking<T, T::Ref, T::MutRef> for DiskVec<T>
where
    T: DiskVecItem,
{
    fn len(&self) -> usize {
        DiskVec::len(self)
    }

    fn push(&mut self, item: T) {
        DiskVec::push(self, &item);
    }

    fn index(&self, index: usize) -> T::Ref {
        &self[index]
    }

    fn index_mut(&mut self, index: usize) -> T::MutRef {
        &mut self[index]
    }
}