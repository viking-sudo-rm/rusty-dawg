// A simplified interface for accessing tokens compared to VecBacking.

use serde::de::DeserializeOwned;
use serde::Serialize;

use graph::memory_backing::disk_backing::disk_vec::DiskVec;

pub trait TokenBacking<T> {
    fn len(&self) -> usize;

    fn get(&self, index: usize) -> T;
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
}