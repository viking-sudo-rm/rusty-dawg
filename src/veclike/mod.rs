pub mod byte_field;

use serde::{Serialize, Deserialize};

use veclike::byte_field::ByteField;

pub trait Veclike<T>
where T: Sized + Serialize + for<'a> Deserialize<'a> + Copy {

    fn len(&self) -> usize;

    fn push(&mut self, item: T);

    fn get(&self, index: usize) -> T;

    fn set(&mut self, index: usize, item: T);

}

impl<T> Veclike<T> for Vec<T>
where T: Sized + Serialize + for<'a> Deserialize<'a> + Copy {

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