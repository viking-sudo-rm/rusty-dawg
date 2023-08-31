use bincode::{serialize, deserialize};
use std::mem::size_of;
use serde::{Serialize, Deserialize};

use veclike::Veclike;

pub mod byte_vec;

pub trait ByteField {

    fn len(&self) -> usize;
    
    fn extend(&mut self, incr: usize);

    fn get_byte(&self, index: usize) -> u8;

    fn set_byte(&mut self, index: usize, value: u8);

}

// We can't have generic types inside methods for a Boxable type.

pub fn get_object<T: Sized + Serialize + for<'a> Deserialize<'a>>(bf: &dyn ByteField, index: usize) -> T {
    let size = size_of::<T>();
    let bytes: Vec<_> = (index..index + size).map(|idx| bf.get_byte(idx)).collect();
    deserialize(&bytes).unwrap()
}

pub fn set_object<T: Sized + Serialize + for<'a> Deserialize<'a>>(bf: &mut dyn ByteField, index: usize, value: T) {
    let bytes: Vec<_> = serialize(&value).unwrap();
    for (idx, byte) in bytes.iter().enumerate() {
        bf.set_byte(index + idx, *byte);
    }
}

impl<T, B: ByteField> Veclike<T> for B
where T: Sized + Serialize + for<'a> Deserialize<'a> + Copy {

    fn len(&self) -> usize {
        ByteField::len(self) / size_of::<T>()
    }

    fn push(&mut self, item: T) {
        let len = Veclike::<T>::len(self);
        let size = size_of::<T>();
        self.extend(size);
        self.set(len, item);
    }

    fn get(&self, index: usize) -> T {
        let size = size_of::<T>();
        get_object(&*self, index * size)
    }

    fn set(&mut self, index: usize, item: T) {
        let size = size_of::<T>();
        set_object(&mut *self, index * size, item);
    }

}