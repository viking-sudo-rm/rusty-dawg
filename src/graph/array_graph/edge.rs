use crate::graph::indexing::{DefaultIx, IndexType, NodeIndex};
use crate::graph::traits::EdgeRef;
use serde::{Deserialize, Serialize};
use std::clone::Clone;

#[derive(Serialize, Deserialize, Default, Copy)]
pub struct ArrayEdge<E, Ix = DefaultIx> {
    #[serde(bound(
        serialize = "E: Serialize, Ix: Serialize",
        deserialize = "E: Deserialize<'de>, Ix: Deserialize<'de>",
    ))]
    pub weight: E,
    pub target: NodeIndex<Ix>,
}

impl<E, Ix> ArrayEdge<E, Ix>
where
    Ix: IndexType + Copy,
    E: Copy,
{
    pub fn new(weight: E, target: NodeIndex<Ix>) -> Self {
        Self { weight, target }
    }
}

impl<E, Ix> Clone for ArrayEdge<E, Ix>
where
    E: Clone,
    Ix: Clone,
{
    fn clone(&self) -> Self {
        ArrayEdge {
            weight: self.weight.clone(),
            target: self.target.clone(),
        }
    }
}

// We can use an Edge object as a "reference" to data on disk.
impl<E, Ix> EdgeRef<E, Ix> for ArrayEdge<E, Ix>
where
    Ix: IndexType + Copy,
    E: Copy,
{
    fn get_weight(self) -> E {
        self.weight
    }

    fn get_target(self) -> NodeIndex<Ix> {
        self.target
    }
}

// We can use a pointer to an Edge object as a reference to data in RAM.
// FIXME(#52): Probably should not be allowing unsafe pointer derefs
impl<E, Ix> EdgeRef<E, Ix> for *const ArrayEdge<E, Ix>
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
