use crate::byte_field::ByteField;

impl ByteField for Vec<u8> {

    fn get(&self, index: usize) -> u8 {
        self[index]
    }

    fn set(&mut self, index: usize, value: u8) {
        self[index] = value;
    }

}

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use byte_field::{ByteField, get_object, set_object};
    use byte_field::byte_field_for_vec;

    #[test]
    fn test_byte_field_for_vec() {
        let bytes: Vec<u8> = vec![0, 2, 5, 57, 123];
        let mut field: Box<dyn ByteField> = Box::new(bytes);
        assert_eq!(field.get(1), 2);
        field.set(1, 43);
        assert_eq!(field.get(1), 43);
    }

    #[test]
    fn test_byte_field_for_vec_get_object() {
        let bytes: Vec<u8> = vec![0, 2, 5, 57, 123];
        let mut field: Box<dyn ByteField> = Box::new(bytes);
        let mut number: u16 = get_object(&*field, 1);
        assert_eq!(number, 256 * 5 + 2);
        set_object(&mut *field, 1, 59);
        number = get_object(&*field, 1);
        assert_eq!(number, 59);
        assert_eq!(field.get(1), 59);
        assert_eq!(field.get(2), 0);
    }
}