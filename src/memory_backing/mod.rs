// pub mod byte_field;
mod vec;

pub trait MemoryBacking<T> {
    fn new() -> Self
    where
        Self: Sized;

    fn with_capacity(size: usize) -> Self
    where
        Self: Sized;

    fn len(&self) -> usize;

    fn push(&mut self, item: T);

    fn index(&self, index: usize) -> &T;

    fn index_mut(&mut self, index: usize) -> &mut T;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
