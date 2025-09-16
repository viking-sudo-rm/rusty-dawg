use serde::{Deserialize, Serialize};
use std::clone::Clone;
use std::marker::Copy;

use crate::graph::indexing::{DefaultIx, EdgeIndex, IndexType, NodeIndex};
use crate::graph::traits::NodeRef;
use crate::weight::Weight;

#[derive(Deserialize, Serialize, Copy, Default)]
pub struct AvlNode<N, Ix = DefaultIx> {
    #[serde(bound(
        serialize = "N: Serialize, Ix: Serialize",
        deserialize = "N: Deserialize<'de>, Ix: Deserialize<'de>",
    ))]
    pub weight: N,
    pub first_edge: EdgeIndex<Ix>,
}

impl<N, Ix> AvlNode<N, Ix>
where
    Ix: IndexType + Copy,
{
    pub fn new(weight: N) -> Self {
        Self {
            weight,
            first_edge: EdgeIndex::end(),
        }
    }
}

impl<N, Ix> Clone for AvlNode<N, Ix>
where
    N: Clone,
    Ix: Clone,
{
    fn clone(&self) -> Self {
        AvlNode {
            weight: self.weight.clone(),
            first_edge: self.first_edge.clone(),
        }
    }
}

// We can use a Node object as a "reference" to data on disk.
impl<N, Ix> NodeRef<N, Ix> for AvlNode<N, Ix>
where
    Ix: IndexType,
    N: Weight,
{
    fn get_weight(self) -> N
    where
        N: Clone,
    {
        self.weight.clone()
    }

    fn get_length(self) -> u64 {
        self.weight.get_length()
    }

    fn get_failure(self) -> Option<NodeIndex<Ix>> {
        // Slightly hacky approach to handle a NodeIndex with non-default Ix.
        self.weight
            .get_failure()
            .map(|phi| NodeIndex::new(phi.index()))
    }

    fn get_count(self) -> usize {
        // FIXME: The count is actually stored in u16.
        self.weight.get_count()
    }

    fn get_first_edge(self) -> EdgeIndex<Ix> {
        self.first_edge
    }
}

// FIXME(#52): We probably should not be allowing these clippy warnings but works for now :/
impl<N, Ix> NodeRef<N, Ix> for *const AvlNode<N, Ix>
where
    Ix: IndexType,
    N: Weight,
{
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn get_weight(self) -> N
    where
        N: Clone,
    {
        unsafe { (*self).weight.clone() }
    }

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn get_length(self) -> u64 {
        unsafe { (*self).weight.get_length() }
    }

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn get_failure(self) -> Option<NodeIndex<Ix>> {
        // Slightly hacky approach to handle a NodeIndex with non-default Ix.
        unsafe {
            (*self)
                .weight
                .get_failure()
                .map(|phi| NodeIndex::new(phi.index()))
        }
    }

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn get_count(self) -> usize {
        unsafe { (*self).weight.get_count() }
    }

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn get_first_edge(self) -> EdgeIndex<Ix> {
        unsafe { (*self).first_edge }
    }
}

pub trait AvlNodeMutRef<Ix> {
    fn set_length(self, length: u64);
    fn set_failure(self, state: Option<NodeIndex<Ix>>);
    fn increment_count(self);
    fn set_count(self, count: usize);
    fn set_first_edge(self, first_edge: EdgeIndex<Ix>);
}

impl<N, Ix> AvlNodeMutRef<Ix> for *mut AvlNode<N, Ix>
where
    Ix: IndexType,
    N: Weight,
{
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn set_length(self, length: u64) {
        unsafe {
            (*self).weight.set_length(length);
        }
    }

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn set_failure(self, state: Option<NodeIndex<Ix>>) {
        // Slightly hacky approach to handle a NodeIndex with non-default Ix.
        unsafe {
            match state {
                Some(q) => (*self).weight.set_failure(Some(NodeIndex::new(q.index()))),
                None => (*self).weight.set_failure(None),
            };
        }
    }

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn increment_count(self) {
        unsafe {
            (*self).weight.increment_count();
        }
    }

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn set_count(self, count: usize) {
        unsafe {
            (*self).weight.set_count(count);
        }
    }

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn set_first_edge(self, first_edge: EdgeIndex<Ix>) {
        unsafe {
            (*self).first_edge = first_edge;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::weight::DefaultWeight;
    use bincode;
    use bincode::{deserialize, serialize, Options};

    #[test]
    fn test_serialize_deserialize_node() {
        type NodeType = AvlNode<DefaultWeight, DefaultIx>;
        let node: NodeType = AvlNode::new(DefaultWeight::new(42, Some(NodeIndex::new(2)), 2));
        let bytes = serialize(&node).unwrap();
        let new_node: NodeType = deserialize(&bytes).unwrap();
        assert_eq!(node.get_length(), new_node.get_length());
        assert_eq!(node.get_failure(), new_node.get_failure());
        assert_eq!(node.get_count(), new_node.get_count());
    }

    #[test]
    fn test_serialize_deserialize_node_with_fixint() {
        type T = AvlNode<DefaultWeight, DefaultIx>;
        let node: T = AvlNode::new(DefaultWeight::new(42, Some(NodeIndex::new(2)), 2));
        let bytes = bincode::DefaultOptions::new()
            .with_fixint_encoding()
            .serialize(&node)
            .unwrap();
        let new_node = bincode::DefaultOptions::new()
            .with_fixint_encoding()
            .deserialize::<T>(&bytes)
            .unwrap();
        assert_eq!(node.get_length(), new_node.get_length());
        assert_eq!(node.get_failure(), new_node.get_failure());
        assert_eq!(node.get_count(), new_node.get_count());
    }
}
