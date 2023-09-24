use graph::memory_backing::disk_backing::disk_vec::DiskVec;
use graph::avl_graph::node::{Node, NodeMutRef};
use graph::avl_graph::edge::EdgeMutRef;
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
        self.disk_vec.set(self.index, &node);
    }

    fn set_failure(self, state: Option<NodeIndex<Ix>>) {
        let mut node = self.disk_vec.get(self.index).unwrap();
        // Handle potential mismatch in Ix.
        let fail_state = match state {
            Some(phi) => Some(NodeIndex::new(phi.index())),
            None => None,
        };
        node.weight.set_failure(fail_state);
        self.disk_vec.set(self.index, &node);
    }

    fn increment_count(self) {
        let mut node = self.disk_vec.get(self.index).unwrap();
        node.weight.increment_count();
        self.disk_vec.set(self.index, &node);
    }

    fn set_first_edge(self, first_edge: EdgeIndex<Ix>) {
        let mut node = self.disk_vec.get(self.index).unwrap();
        node.first_edge = first_edge;
        self.disk_vec.set(self.index, &node);
    }
}