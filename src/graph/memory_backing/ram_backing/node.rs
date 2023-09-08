use serde::{Deserialize, Serialize};
use std::clone::Clone;
use std::marker::Copy;

use graph::indexing::{DefaultIx, EdgeIndex, IndexType};
use graph::memory_backing::node_backing::NodeBacking;

#[derive(Deserialize, Serialize, Copy)]
pub struct Node<N, Ix = DefaultIx> {
    #[serde(bound(
        serialize = "N: Serialize, Ix: Serialize",
        deserialize = "N: Deserialize<'de>, Ix: Deserialize<'de>",
    ))]
    weight: N,
    first_edge: EdgeIndex<Ix>,
}

impl<N, Ix> NodeBacking<N, Ix> for Node<N, Ix>
where
    Ix: Copy,
{
    fn get_weight(&self) -> &N {
        &self.weight
    }

    fn get_weight_mut(&mut self) -> &mut N {
        &mut self.weight
    }

    fn get_first_edge(&self) -> EdgeIndex<Ix> {
        self.first_edge
    }

    fn set_first_edge(&mut self, first_edge: EdgeIndex<Ix>) {
        self.first_edge = first_edge;
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

impl<N, Ix: IndexType> Node<N, Ix> {
    pub fn new(weight: N) -> Self {
        Self {
            weight,
            first_edge: EdgeIndex::end(),
        }
    }
}
