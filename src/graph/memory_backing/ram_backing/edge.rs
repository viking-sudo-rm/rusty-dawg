use serde::{Deserialize, Serialize};
use std::clone::Clone;

use graph::indexing::{DefaultIx, EdgeIndex, IndexType, NodeIndex};
use graph::memory_backing::EdgeBacking;

#[derive(Serialize, Deserialize)]
pub struct Edge<E, Ix = DefaultIx> {
    #[serde(bound(
        serialize = "E: Serialize, Ix: Serialize",
        deserialize = "E: Deserialize<'de>, Ix: Deserialize<'de>",
    ))]
    pub weight: E,
    pub target: NodeIndex<Ix>,
    pub left: EdgeIndex<Ix>,
    pub right: EdgeIndex<Ix>,
    pub balance_factor: i8,
}

impl<E, Ix> Clone for Edge<E, Ix>
where
    E: Clone,
    Ix: Clone,
{
    fn clone(&self) -> Self {
        Edge {
            weight: self.weight.clone(),
            target: self.target.clone(),
            left: self.left.clone(),
            right: self.right.clone(),
            balance_factor: self.balance_factor,
        }
    }
}

impl<E, Ix: IndexType> Edge<E, Ix> {
    pub fn new(weight: E, target: NodeIndex<Ix>) -> Self {
        Edge {
            weight,
            target,
            left: EdgeIndex::end(),
            right: EdgeIndex::end(),
            balance_factor: 0,
        }
    }
}

impl<E, Ix> EdgeBacking<E, Ix> for Edge<E, Ix>
where
    Ix: Copy,
{
    fn get_weight(&self) -> &E {
        &self.weight
    }

    fn get_target(&self) -> NodeIndex<Ix> {
        self.target
    }

    fn set_target(&mut self, target: NodeIndex<Ix>) {
        self.target = target;
    }

    fn get_left(&self) -> EdgeIndex<Ix> {
        self.left
    }

    fn set_left(&mut self, left: EdgeIndex<Ix>) {
        self.left = left;
    }

    fn get_right(&self) -> EdgeIndex<Ix> {
        self.right
    }

    fn set_right(&mut self, right: EdgeIndex<Ix>) {
        self.right = right;
    }

    fn get_balance_factor(&self) -> i8 {
        self.balance_factor
    }

    fn set_balance_factor(&mut self, bf: i8) {
        self.balance_factor = bf;
    }
}
