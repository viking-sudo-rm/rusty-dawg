// pub mod byte_field;
mod vec;

use std::ops::{Index, IndexMut};

pub trait MemoryBacking<T> {

    fn new() -> Self where Self: Sized;

    fn with_capacity(size: usize) -> Self where Self: Sized;

    fn len(&self) -> usize;

    fn push(&mut self, item: T);

    fn index(&self, index: usize) -> &T;

    fn index_mut(&mut self, index: usize) -> &mut T;

}

impl<T> Index<usize> for dyn MemoryBacking<T> {
    type Output = T;

    fn index(&self, index: usize) -> &T {
        self.index(index)
    }
}

impl<T> IndexMut<usize> for dyn MemoryBacking<T> {
    fn index_mut(&mut self, index: usize) -> &mut T {
        self.index_mut(index)
    }
}