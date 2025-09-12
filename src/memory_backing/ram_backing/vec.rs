use crate::memory_backing::{InternallyImmutableVecBacking, VecBacking};

// FIXME: Did this with unsafe pointers for convenience but would be good to use &/&mut!

impl<T> InternallyImmutableVecBacking<T> for Vec<T> {
    type TRef = *const T;

    fn len(&self) -> usize {
        Vec::len(self)
    }

    fn index(&self, index: usize) -> Self::TRef {
        &self[index]
    }

    fn set(&mut self, index: usize, value: T) {
        self[index] = value;
    }

    fn push(&mut self, item: T) {
        Vec::push(self, item);
    }
}
impl<T> VecBacking<T> for Vec<T> {
    type TMutRef = *mut T;

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
