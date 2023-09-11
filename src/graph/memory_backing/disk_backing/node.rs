use serde::{Deserialize, Serialize};
use std::clone::Clone;
use std::mem::size_of;
use std::marker::Copy;

use graph::indexing::{DefaultIx, EdgeIndex, IndexType};
use graph::memory_backing::node_backing::NodeBacking;

pub struct Node<N, Ix = DefaultIx> {
    pub bytes: DiskVec,  // FIXME: not sure what the right type is here.
    pub idx: usize,  // Initialize to -1 to mark not pushed?
}

const WEIGHT_START: usize = 0;
const EDGE_START: usize = size_of::<N>();
const END: usize = size_of::<N>() + size_of::<EdgeIndex<Ix>>();

impl<N, Ix> NodeBacking<N, Ix> for Node<N, Ix>
where
    Ix: IndexType + Copy,
{
    fn get_weight(&self) -> &N {
        // FIXME: need to adapt this (idea: read bytes from WEIGHT_START to TARGET_START)
        let bytes = self.bytes.read(WEIGHT_START, TARGET_START);
        let weight = deserialize(&bytes).unwrap()
        // FIXME: compiler error because there is dangling pointer. fixes:
        //  1. refactor return type to E (easy-ish)
        //  2. create a new ZeroCopy weight type and return one of those (more useful for Node)
        &weight
    }

    fn get_weight_mut(&mut self) -> &mut N {
        // FIXME: ???
        // Idea: the weight type (N) needs to be zero-copy too?
    }

    fn get_first_edge(&self) -> EdgeIndex<Ix> {
        let bytes = self.bytes.read(EDGE_START, END);
        deserialize(&bytes).unwrap()
    }

    fn set_first_edge(&mut self, first_edge: EdgeIndex<Ix>) {
        let bytes: Vec<_> = serialize(&first_edge).unwrap();
        // FIXME: need to adapt this (idea: write bytes starting at EDGE_START)
        self.bytes.write(bytes, EDGE_START);
    }
}
