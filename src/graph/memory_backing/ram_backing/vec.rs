use graph::memory_backing::vec_backing::VecBacking;

impl<T> VecBacking<T, *mut T> for Vec<T> {
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

    fn index(&self, index: usize) -> &T {
        &self[index]
    }

    fn index_mut(&mut self, index: usize) -> *mut T {
        &mut self[index]
    }
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use graph::memory_backing::vec_backing::VecBacking;

    #[test]
    fn test_index() {
        let mb: Box<dyn VecBacking<u8, *mut u8>> = Box::new(vec![12, 21]);
        assert_eq!(*mb.index(0), 12);
        assert_eq!(*mb.index(1), 21);
    }

    #[test]
    fn test_index_mut() {
        let mut mb: Box<dyn VecBacking<u8, *mut u8>> = Box::new(vec![12, 21]);
        assert_eq!(*mb.index(0), 12);
        unsafe {
            *mb.index_mut(0) = 32;
        }
        assert_eq!(*mb.index(0), 32);
    }
}
