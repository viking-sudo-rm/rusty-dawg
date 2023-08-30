use bincode::{serialize, deserialize};
use std::mem::size_of;
use serde::{Serialize, Deserialize};

pub mod byte_field_for_vec;

pub trait ByteField {

    fn get(&self, index: usize) -> u8;

    fn set(&mut self, index: usize, value: u8);

}

// We can't have generic types inside methods for a Boxable type.

pub fn get_object<T: Sized + Serialize + for<'a> Deserialize<'a>>(bf: &dyn ByteField, index: usize) -> T {
    let size = size_of::<T>();
    let bytes: Vec<_> = (index..index + size).map(|idx| bf.get(idx)).collect();
    deserialize(&bytes).unwrap()
}

pub fn set_object<T: Sized + Serialize + for<'a> Deserialize<'a>>(bf: &mut dyn ByteField, index: usize, value: T) {
    let bytes: Vec<_> = serialize(&value).unwrap();
    for (idx, byte) in bytes.iter().enumerate() {
        bf.set(index + idx, *byte);
    }
}