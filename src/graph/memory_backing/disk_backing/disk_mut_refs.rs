use graph::memory_backing::disk_backing::disk_vec::DiskVec;
use graph::avl_graph::node::{Node, NodeMutRef};
use graph::avl_graph::edge::{Edge, EdgeMutRef};
use graph::memory_backing::disk_backing::{NodeIndex, EdgeIndex, IndexType};
use serde::{Serialize, Deserialize};
use serde::de::DeserializeOwned;
use weight::Weight;

pub struct DiskNodeMutRef<'a, N, Ix> {
    disk_vec: &'a mut DiskVec<Node<N, Ix>>,
    index: usize,
}

// TODO: Only overwrite the specific field in the DiskVec rather than read/write.
impl<'a, N, Ix> NodeMutRef<Ix> for DiskNodeMutRef<'a, N, Ix> where
    Ix: IndexType,
    N: Weight,
    Node<N, Ix>: Serialize + DeserializeOwned + Default,
{
    fn set_length(self, length: u64) {
        let mut node = self.disk_vec.get(self.index).unwrap();
        node.weight.set_length(length);
        let _ = self.disk_vec.set(self.index, &node);
    }

    fn set_failure(self, state: Option<NodeIndex<Ix>>) {
        let mut node = self.disk_vec.get(self.index).unwrap();
        // Handle potential mismatch in Ix.
        let fail_state = match state {
            Some(phi) => Some(NodeIndex::new(phi.index())),
            None => None,
        };
        node.weight.set_failure(fail_state);
        let _ = self.disk_vec.set(self.index, &node);
    }

    fn increment_count(self) {
        let mut node = self.disk_vec.get(self.index).unwrap();
        node.weight.increment_count();
        let _ = self.disk_vec.set(self.index, &node);
    }

    fn set_first_edge(self, first_edge: EdgeIndex<Ix>) {
        let mut node = self.disk_vec.get(self.index).unwrap();
        node.first_edge = first_edge;
        let _ = self.disk_vec.set(self.index, &node);
    }
}

pub struct DiskEdgeMutRef<'a, E, Ix> {
    disk_vec: &'a mut DiskVec<Edge<E, Ix>>,
    index: usize,
}

impl<'a, E, Ix> EdgeMutRef<Ix> for DiskEdgeMutRef<'a, E, Ix>
where
    Ix: IndexType + Copy,
    Edge<E, Ix>: Serialize + DeserializeOwned + Default,
{
    fn set_target(self, target: NodeIndex<Ix>) {
        let mut edge = self.disk_vec.get(self.index).unwrap();
        edge.target = target;
        let _ = self.disk_vec.set(self.index, &edge);
    }

    fn set_left(self, left: EdgeIndex<Ix>) {
        let mut edge = self.disk_vec.get(self.index).unwrap();
        edge.left = left;
        let _ = self.disk_vec.set(self.index, &edge);
    }

    fn set_right(self, right: EdgeIndex<Ix>) {
        let mut edge = self.disk_vec.get(self.index).unwrap();
        edge.right = right;
        let _ = self.disk_vec.set(self.index, &edge);
    }

    fn set_balance_factor(self, bf: i8) {
        let mut edge = self.disk_vec.get(self.index).unwrap();
        edge.balance_factor = bf;
        let _ = self.disk_vec.set(self.index, &edge);
    }
}

pub trait DiskVecItem: Sized {
    type MutRef<'a> where Self: 'a;

    fn new_mut_ref<'a>(disk_vec: &'a mut DiskVec<Self>, index: usize) -> Self::MutRef<'a>;
}

impl<N, Ix> DiskVecItem for Node<N, Ix> {
    type MutRef<'a> = DiskNodeMutRef<'a, N, Ix> where N: 'a, Ix: 'a;

    fn new_mut_ref<'a>(disk_vec: &'a mut DiskVec<Node<N, Ix>>, index: usize) -> Self::MutRef<'a> {
        DiskNodeMutRef {disk_vec, index}
    }
}

impl<E, Ix> DiskVecItem for Edge<E, Ix> {
    type MutRef<'a> = DiskEdgeMutRef<'a, E, Ix> where E: 'a, Ix: 'a;

    fn new_mut_ref<'a>(disk_vec: &'a mut DiskVec<Edge<E, Ix>>, index: usize) -> Self::MutRef<'a> {
        DiskEdgeMutRef { disk_vec, index }
    }
}