use memory_backing::MemoryBacking;
use serde::{Serialize, Deserialize};

impl<T> MemoryBacking<T> for Vec<T>
where T: Sized + Serialize + for<'a> Deserialize<'a> + Copy {

    fn new() -> Self {
        Vec::new()
    }

    fn with_capacity(size: usize) -> Self {
        Vec::with_capacity(size)
    }

    fn len(&self) -> usize {
        Vec::len(self)
    }

    fn push(&mut self, item: T) {
        Vec::push(self, item);
    }

    fn get(&self, index: usize) -> T {
        self[index]
    }

    fn set(&mut self, index: usize, item: T) {
        self[index] = item;
    }

}