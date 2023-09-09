use std::clone::Clone;
use std::mem::size_of;
use bincode::{serialize, deserialize};

use graph::indexing::{DefaultIx, EdgeIndex, IndexType, NodeIndex};
use graph::memory_backing::EdgeBacking;

const WEIGHT_START: usize = 0;
const TARGET_START: usize = size_of::<E>();
const LEFT_START: usize = size_of::<E>() + size_of::<NodeIndex<Ix>>();
const RIGHT_START: usize = size_of::<E>() + size_of::<NodeIndex<Ix>>() + size_of::<EdgeIndex<Ix>>();
const BF_START: usize = size_of::<E>() + size_of::<NodeIndex<Ix>>() + 2 * size_of::<EdgeIndex<Ix>>();
const END: usize = size_of::<E>() + size_of::<NodeIndex<Ix>>() + 2 * size_of::<EdgeIndex<Ix>>() + size_of::<i8>();

pub struct Edge<E, Ix = DefaultIx> {
    pub bytes: File,  // FIXME: not sure what the right type is here.
    pub idx: usize,
}

impl<E, Ix> EdgeBacking<E, Ix> for Edge<E, Ix>
where
    Ix: IndexType + Copy,
    E: Sized,
{
    fn get_weight(&self) -> &E {
        let bytes = self.bytes.read(WEIGHT_START, TARGET_START);
        let weight = deserialize(&bytes).unwrap()
        &weight
    }

    fn get_target(&self) -> NodeIndex<Ix> {
        let bytes = self.bytes.read(TARGET_START, LEFT_START);
        deserialize(&bytes).unwrap()
    }

    fn set_target(&mut self, target: NodeIndex<Ix>) {
        let bytes: Vec<_> = serialize(&target).unwrap();
        self.bytes.write(bytes, TARGET_START);
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
