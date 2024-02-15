use crate::memory_backing::VecBacking;

// FIXME: Did this with unsafe pointers for convenience but would be good to use &/&mut!
impl<T> VecBacking<T> for Vec<T> {
    type TRef = *const T;
    type TMutRef = *mut T;

    fn len(&self) -> usize {
        Vec::len(self)
    }

    fn push(&mut self, item: T) {
        Vec::push(self, item);
    }

    fn index(&self, index: usize) -> Self::TRef {
        &self[index]
    }

    fn index_mut(&mut self, index: usize) -> Self::TMutRef {
        &mut self[index]
    }
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use super::*;

    #[test]
    fn test_index() {
        let mb: Box<dyn VecBacking<u8, TRef = *const u8, TMutRef = *mut u8>> =
            Box::new(vec![12, 21]);
        unsafe {
            assert_eq!(*mb.index(0), 12);
            assert_eq!(*mb.index(1), 21);
        }
    }

    #[test]
    fn test_index_mut() {
        let mut mb: Box<dyn VecBacking<u8, TRef = *const u8, TMutRef = *mut u8>> =
            Box::new(vec![12, 21]);
        unsafe {
            assert_eq!(*mb.index(0), 12);
            *mb.index_mut(0) = 32;
            assert_eq!(*mb.index(0), 32);
        }
    }
}
