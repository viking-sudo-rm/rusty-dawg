use serde::{Deserialize, Serialize};
use std::clone::Clone;

use crate::graph::indexing::{DefaultIx, EdgeIndex, IndexType, NodeIndex};
use crate::graph::traits::EdgeRef;

#[derive(Serialize, Deserialize, Default, Copy)]
pub struct AvlEdge<E, Ix = DefaultIx> {
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

impl<E, Ix> Clone for AvlEdge<E, Ix>
where
    E: Clone,
    Ix: Clone,
{
    fn clone(&self) -> Self {
        AvlEdge {
            weight: self.weight.clone(),
            target: self.target.clone(),
            left: self.left.clone(),
            right: self.right.clone(),
            balance_factor: self.balance_factor,
        }
    }
}

impl<E, Ix> AvlEdge<E, Ix>
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
}

pub trait AvlEdgeRef<E, Ix>: EdgeRef<E, Ix> {
    fn get_left(self) -> EdgeIndex<Ix>;
    fn get_right(self) -> EdgeIndex<Ix>;
    fn get_balance_factor(self) -> i8;
}

impl<E, Ix> EdgeRef<E, Ix> for AvlEdge<E, Ix> {
    fn get_weight(self) -> E {
        self.weight
    }

    fn get_target(self) -> NodeIndex<Ix> {
        self.target
    }
}
// We can use an Edge object as a "reference" to data on disk.
impl<E, Ix> AvlEdgeRef<E, Ix> for AvlEdge<E, Ix>
where
    Ix: IndexType + Copy,
    E: Copy,
{
    fn get_left(self) -> EdgeIndex<Ix> {
        self.left
    }

    fn get_right(self) -> EdgeIndex<Ix> {
        self.right
    }

    fn get_balance_factor(self) -> i8 {
        self.balance_factor
    }
}

// FIXME(#52): Probably should not be allowing unsafe pointer derefs
impl<E, Ix> EdgeRef<E, Ix> for *const AvlEdge<E, Ix>
where
    Ix: IndexType + Copy,
    E: Copy,
{
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn get_weight(self) -> E {
        unsafe { (*self).weight }
    }

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn get_target(self) -> NodeIndex<Ix> {
        unsafe { (*self).target }
    }
}
// We can use a pointer to an Edge object as a reference to data in RAM.
// FIXME(#52): Probably should not be allowing unsafe pointer derefs
impl<E, Ix> AvlEdgeRef<E, Ix> for *const AvlEdge<E, Ix>
where
    Ix: IndexType + Copy,
    E: Copy,
{
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn get_left(self) -> EdgeIndex<Ix> {
        unsafe { (*self).left }
    }

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn get_right(self) -> EdgeIndex<Ix> {
        unsafe { (*self).right }
    }

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn get_balance_factor(self) -> i8 {
        unsafe { (*self).balance_factor }
    }
}

pub trait AvlEdgeMutRef<E, Ix> {
    fn set_weight(self, weight: E);
    fn set_target(self, target: NodeIndex<Ix>);
    fn set_left(self, left: EdgeIndex<Ix>);
    fn set_right(self, right: EdgeIndex<Ix>);
    fn set_balance_factor(self, bf: i8);
}

impl<E, Ix> AvlEdgeMutRef<E, Ix> for *mut AvlEdge<E, Ix>
where
    E: Copy,
    Ix: IndexType + Copy,
{
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn set_weight(self, weight: E) {
        unsafe {
            (*self).weight = weight;
        }
    }

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn set_target(self, target: NodeIndex<Ix>) {
        unsafe {
            (*self).target = target;
        }
    }

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn set_left(self, left: EdgeIndex<Ix>) {
        unsafe {
            (*self).left = left;
        }
    }

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn set_right(self, right: EdgeIndex<Ix>) {
        unsafe {
            (*self).right = right;
        }
    }

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn set_balance_factor(self, bf: i8) {
        unsafe {
            (*self).balance_factor = bf;
        }
    }
}

impl<E, Ix> AvlEdgeMutRef<E, Ix> for &mut AvlEdge<E, Ix>
where
    E: Copy,
    Ix: IndexType + Copy,
{
    fn set_weight(self, weight: E) {
        self.weight = weight;
    }

    fn set_target(self, target: NodeIndex<Ix>) {
        self.target = target;
    }

    fn set_left(self, left: EdgeIndex<Ix>) {
        self.left = left;
    }

    fn set_right(self, right: EdgeIndex<Ix>) {
        self.right = right;
    }

    fn set_balance_factor(self, bf: i8) {
        self.balance_factor = bf;
    }
}
