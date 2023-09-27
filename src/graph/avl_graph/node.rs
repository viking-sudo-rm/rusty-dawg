use serde::{Deserialize, Serialize};
use std::clone::Clone;
use std::marker::Copy;

use graph::indexing::{DefaultIx, EdgeIndex, IndexType, NodeIndex};
use weight::Weight;

#[derive(Deserialize, Serialize, Copy, Default)]
pub struct Node<N, Ix = DefaultIx> {
    #[serde(bound(
        serialize = "N: Serialize, Ix: Serialize",
        deserialize = "N: Deserialize<'de>, Ix: Deserialize<'de>",
    ))]
    pub weight: N,
    pub first_edge: EdgeIndex<Ix>,
}

impl<N, Ix> Node<N, Ix>
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

impl<N, Ix> Clone for Node<N, Ix>
where
    N: Clone,
    Ix: Clone,
{
    fn clone(&self) -> Self {
        Node {
            weight: self.weight.clone(),
            first_edge: self.first_edge.clone(),
        }
    }
}

pub trait NodeRef<N, Ix> {
    fn get_weight(self) -> N
    where
        N: Clone;
    fn get_length(self) -> u64;
    fn get_failure(self) -> Option<NodeIndex<Ix>>;
    fn get_count(self) -> u64;
    fn get_first_edge(self) -> EdgeIndex<Ix>;
}

// We can use a Node object as a "reference" to data on disk.
impl<N, Ix> NodeRef<N, Ix> for Node<N, Ix>
where
    Ix: IndexType,
    N: Weight,
{
    fn get_weight(self) -> N where N: Clone {
        self.weight.clone()
    }

    fn get_length(self) -> u64 {
        self.weight.get_length()
    }

    fn get_failure(self) -> Option<NodeIndex<Ix>> {
        // Slightly hacky approach to handle a NodeIndex with non-default Ix.
        match self.weight.get_failure() {
            Some(phi) => Some(NodeIndex::new(phi.index())),
            None => None,
        }
    }

    fn get_count(self) -> u64 {
        self.weight.get_count()
    }

    fn get_first_edge(self) -> EdgeIndex<Ix> {
        self.first_edge
    }
}

impl<N, Ix> NodeRef<N, Ix> for *const Node<N, Ix>
where
    Ix: IndexType,
    N: Weight,
{
    fn get_weight(self) -> N
    where
        N: Clone,
    {
        unsafe { (*self).weight.clone() }
    }

    fn get_length(self) -> u64 {
        unsafe { (*self).weight.get_length() }
    }

    fn get_failure(self) -> Option<NodeIndex<Ix>> {
        // Slightly hacky approach to handle a NodeIndex with non-default Ix.
        unsafe {
            match (*self).weight.get_failure() {
                Some(phi) => Some(NodeIndex::new(phi.index())),
                None => None,
            }
        }
    }

    fn get_count(self) -> u64 {
        unsafe { (*self).weight.get_count() }
    }

    fn get_first_edge(self) -> EdgeIndex<Ix> {
        unsafe { (*self).first_edge }
    }
}

pub trait NodeMutRef<Ix> {
    fn set_length(self, length: u64);
    fn set_failure(self, state: Option<NodeIndex<Ix>>);
    fn increment_count(self);
    fn set_first_edge(self, first_edge: EdgeIndex<Ix>);
}

impl<N, Ix> NodeMutRef<Ix> for *mut Node<N, Ix>
where
    Ix: IndexType,
    N: Weight,
{
    fn set_length(self, length: u64) {
        unsafe {
            (*self).weight.set_length(length);
        }
    }

    fn set_failure(self, state: Option<NodeIndex<Ix>>) {
        // Slightly hacky approach to handle a NodeIndex with non-default Ix.
        unsafe {
            match state {
                Some(q) => (*self).weight.set_failure(Some(NodeIndex::new(q.index()))),
                None => (*self).weight.set_failure(None),
            };
        }
    }

    fn increment_count(self) {
        unsafe {
            (*self).weight.increment_count();
        }
    }

    fn set_first_edge(self, first_edge: EdgeIndex<Ix>) {
        unsafe {
            (*self).first_edge = first_edge;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::weight::WeightMinimal;

    use super::*;
    use bincode;
    use bincode::{serialize, deserialize, Options};
    use weight::DefaultWeight;

        #[test]
    fn test_serialize_deserialize_node() {
        type NodeType = Node<WeightMinimal, DefaultIx>;
        let node: NodeType = Node::new(DefaultWeight::new(42, Some(NodeIndex::new(2)), 2));
        let bytes = serialize(&node).unwrap();
        let new_node: NodeType = deserialize(&bytes).unwrap();
        assert_eq!(node.get_length(), new_node.get_length());
        assert_eq!(node.get_failure(), new_node.get_failure());
        assert_eq!(node.get_count(), new_node.get_count());
    }

    #[test]
    fn test_serialize_deserialize_node_with_fixint() {
        type T = Node<WeightMinimal, DefaultIx>;
        let node: T = Node::new(DefaultWeight::new(42, Some(NodeIndex::new(2)), 2));
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