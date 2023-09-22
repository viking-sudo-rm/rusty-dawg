use std::cell::RefCell;
use std::clone::Clone;
use std::mem::size_of;

use anyhow::{bail, Result};
use bincode::{deserialize, serialize};

use super::vec::DiskVec;
use graph::indexing::{DefaultIx, EdgeIndex, IndexType, NodeIndex};
use graph::memory_backing::EdgeBacking;

const WEIGHT_START: usize = 0;
const TARGET_START: usize = size_of::<E>();
const LEFT_START: usize = size_of::<E>() + size_of::<NodeIndex<Ix>>();
const RIGHT_START: usize = size_of::<E>() + size_of::<NodeIndex<Ix>>() + size_of::<EdgeIndex<Ix>>();
const BF_START: usize =
    size_of::<E>() + size_of::<NodeIndex<Ix>>() + 2 * size_of::<EdgeIndex<Ix>>();
const END: usize =
    size_of::<E>() + size_of::<NodeIndex<Ix>>() + 2 * size_of::<EdgeIndex<Ix>>() + size_of::<i8>();

struct MinimalEdge<E, Ix = DefaultIx> {
    weight: E,
    target: NodeIndex<Ix>,
}

pub struct Edge<E, Ix = DefaultIx> {
    pub(crate) vec: Option<DiskVec<MinimalEdge<E, Ix>>>, // Initialize to None, set to Some when pushed.
    pub(crate) idx: Option<usize>, // Initialize to -1, set to index when pushed.
    // Only used to store data when an Edge is created before it's pushed.
    fields: MinimalEdge<E, Ix>,
}

impl<E, Ix> Edge<E, Ix> {
    fn ensure_fields(&self) -> Result<()> {
        if self.fields.borrow().is_some() {
            Ok(())
        } else {
            if self.vec.is_none() || self.idx.is_none() {
                bail!("DiskVec has not been assigned to Edge!");
            }
            let fields = self.fields.borrow_mut();
            *fields = self.vec.unwrap().get(self.idx.unwrap());
            Ok(())
        }
    }

    fn save_fields(&self) -> Result<()> {
        if self.vec.is_none() || self.idx.is_none() {
            bail!("DiskVec has not been assigned to Edge!");
        }
        let fields = self.fields.borrow();
        if fields.is_none() {
            bail!("fields have not been set on Edge!");
        }
        self.vec.set(self.idx.unwrap(), fields.unwrap())?;
        Ok(())
    }
}

impl<E, Ix> EdgeBacking<E, Ix> for Edge<E, Ix>
where
    Ix: IndexType + Copy,
    E: Sized,
{
    fn new(weight: E, target: NodeIndex<Ix>) -> Self {
        Self {
            vec: None,
            idx: None,
            fields: RefCell::new(MinimalEdge { weight, target }),
        }
    }

    fn get_weight(&self) -> &E {
        &self.fields.weight
    }

    fn get_target(&self) -> NodeIndex<Ix> {
        self.fields.target
    }

    fn set_target(&mut self, target: NodeIndex<Ix>) {
        self.fields.target = target;
        self.vec.unwrap().set(self.idx.unwrap(), &self.fields);
    }

    fn get_left(&self) -> EdgeIndex<Ix> {
        let bytes = self.bytes.read(LEFT_START, RIGHT_START);
        deserialize(&bytes).unwrap()
    }

    fn set_left(&mut self, left: EdgeIndex<Ix>) {
        let bytes: Vec<_> = serialize(&left).unwrap();
        self.bytes.write(bytes, LEFT_START);
    }

    fn get_right(&self) -> EdgeIndex<Ix> {
        let bytes = self.bytes.read(RIGHT_START, BF_START);
        deserialize(&bytes).unwrap()
    }

    fn set_right(&mut self, right: EdgeIndex<Ix>) {
        let bytes: Vec<_> = serialize(&right).unwrap();
        self.bytes.write(bytes, RIGHT_START);
    }

    fn get_balance_factor(&self) -> i8 {
        let bytes = self.bytes.read(BF_START, END);
        deserialize(&bytes).unwrap()
    }

    fn set_balance_factor(&mut self, bf: i8) {
        let bytes: Vec<_> = serialize(&bf).unwrap();
        self.bytes.write(bytes, BF_START);
    }
}
