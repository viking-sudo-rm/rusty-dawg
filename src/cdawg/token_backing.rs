// A simplified interface for accessing tokens compared to VecBacking.

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::graph::memory_backing::disk_backing::disk_vec::DiskVec;

pub trait TokenBacking<T> {
    fn len(&self) -> usize;

    fn get(&self, index: usize) -> T;

    fn push(&mut self, value: T);
}

impl<T> TokenBacking<T> for Vec<T>
where T: Copy
{
    fn len(&self) -> usize {
        Vec::len(self)
    }

    fn get(&self, index: usize) -> T {
        self[index]
    }

    fn push(&mut self, value: T) {
        Vec::push(self, value);
    }
}

impl<T> TokenBacking<T> for DiskVec<T>
where
    T: Serialize + DeserializeOwned + Default,
{
    fn len(&self) -> usize {
        DiskVec::len(self)
    }

    fn get(&self, index: usize) -> T {
        DiskVec::get(self, index).unwrap()
    }

    fn push(&mut self, value: T) {
        let _ = DiskVec::push(self, &value);
    }
}