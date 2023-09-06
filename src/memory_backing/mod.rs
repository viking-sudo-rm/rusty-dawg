pub mod byte_field;
mod vec;

use serde::{Serialize, Deserialize};

pub trait MemoryBacking<T>
where T: Sized + Serialize + for<'a> Deserialize<'a> + Copy {

    fn new() -> Self where Self: Sized;

    fn with_capacity(size: usize) -> Self where Self: Sized;

    fn len(&self) -> usize;

    fn push(&mut self, item: T);

    fn get(&self, index: usize) -> T;

    fn set(&mut self, index: usize, item: T);

}