use serde::{Deserialize, Serialize};
use std::clone::Clone;

use graph::indexing::{DefaultIx, EdgeIndex, IndexType, NodeIndex};

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

impl<E, Ix> Edge<E, Ix>
where
    Ix: IndexType + Copy,
    E: Copy,
{
    pub fn new(weight: E, target: NodeIndex<Ix>) -> Self {
        Self {
            weight,
            target,
            left: EdgeIndex::end(),
            right: EdgeIndex::end(),
            balance_factor: 0,
        }
    }

    pub fn get_weight(&self) -> E {
        self.weight
    }

    pub fn get_target(&self) -> NodeIndex<Ix> {
        self.target
    }

    pub fn get_left(&self) -> EdgeIndex<Ix> {
        self.left
    }

    pub fn get_right(&self) -> EdgeIndex<Ix> {
        self.right
    }

    pub fn get_balance_factor(&self) -> i8 {
        self.balance_factor
    }
}

pub trait EdgeRef<E, Ix> {
    fn get_weight(self) -> E;
    fn get_target(self) -> NodeIndex<Ix>;
    fn get_left(self) -> EdgeIndex<Ix>;
    fn get_right(self) -> EdgeIndex<Ix>;
    fn get_balance_factor(self) -> i8;
}

impl<E, Ix> EdgeRef<E, Ix> for *const Edge<E, Ix>
where
    Ix: IndexType + Copy,
    E: Copy,
{
    fn get_weight(self) -> E {
        unsafe { Edge::get_weight(&*self) }
    }

    fn get_target(self) -> NodeIndex<Ix> {
        unsafe { Edge::get_target(&*self) }
    }

    fn get_left(self) -> EdgeIndex<Ix> {
        unsafe { Edge::get_left(&*self) }
    }

    fn get_right(self) -> EdgeIndex<Ix> {
        unsafe { Edge::get_right(&*self) }
    }

    fn get_balance_factor(self) -> i8 {
        unsafe { Edge::get_balance_factor(&*self) }
    }
}

pub trait EdgeMutRef<Ix> {
    fn set_target(self, target: NodeIndex<Ix>);
    fn set_left(self, left: EdgeIndex<Ix>);
    fn set_right(self, right: EdgeIndex<Ix>);
    fn set_balance_factor(self, bf: i8);
}

impl<E, Ix> EdgeMutRef<Ix> for *mut Edge<E, Ix>
where
    Ix: IndexType + Copy,
{
    fn set_target(self, target: NodeIndex<Ix>) {
        unsafe {
            (*self).target = target;
        }
    }

    fn set_left(self, left: EdgeIndex<Ix>) {
        unsafe {
            (*self).left = left;
        }
    }

    fn set_right(self, right: EdgeIndex<Ix>) {
        unsafe {
            (*self).right = right;
        }
    }

    fn set_balance_factor(self, bf: i8) {
        unsafe {
            (*self).balance_factor = bf;
        }
    }
}
