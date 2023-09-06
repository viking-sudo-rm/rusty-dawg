use memory_backing::byte_field::ByteField;

pub struct ByteVec {
    bytes: Vec<u8>,
}

impl ByteField for ByteVec {

    fn new() -> Self {
        Self { bytes: Vec::new() }
    }

    fn with_capacity(size: usize) -> Self {
        Self { bytes: Vec::with_capacity(size) }
    }

    fn len(&self) -> usize {
        self.bytes.len()
    }

    fn extend(&mut self, incr: usize) {
        // This might be slow compared to just changing capacity?
        let len = self.bytes.len();
        self.bytes.resize(len + incr, 0);
    }

    fn get_byte(&self, index: usize) -> u8 {
        self.bytes[index]
    }

    fn set_byte(&mut self, index: usize, value: u8) {
        self.bytes[index] = value;
    }

}

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use memory_backing::MemoryBacking;
    use memory_backing::byte_field::{ByteField, get_object, set_object};
    use memory_backing::byte_field::byte_vec::*;

    #[test]
    fn test_byte_field_for_vec() {
        let bytes = ByteVec {bytes: vec![0, 2, 5, 57, 123]};
        let mut field: Box<dyn ByteField> = Box::new(bytes);
        assert_eq!(field.get_byte(1), 2);
        field.set_byte(1, 43);
        assert_eq!(field.get_byte(1), 43);
    }

    #[test]
    fn test_byte_field_for_vec_get_object() {
        let bytes = ByteVec {bytes: vec![0, 2, 5, 57, 123]};
        let mut field: Box<dyn ByteField> = Box::new(bytes);
        let mut number: u16 = get_object(&*field, 1);
        assert_eq!(number, 256 * 5 + 2);
        set_object(&mut *field, 1, 59);
        number = get_object(&*field, 1);
        assert_eq!(number, 59);
        assert_eq!(field.get_byte(1), 59);
        assert_eq!(field.get_byte(2), 0);
    }

    #[test]
    fn test_byte_field_as_mb() {
        let bytes = ByteVec {bytes: vec![0, 2, 5, 57, 123, 0]};
        let mut mb: Box<dyn MemoryBacking<u16>> = Box::new(bytes);
        assert_eq!(mb.get(0), 512);
        mb.set(0, 432);
        assert_eq!(mb.get(0), 432);
    }
}