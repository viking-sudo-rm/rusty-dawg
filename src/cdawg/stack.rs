use crate::memory_backing::DiskVec;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub trait Stack<T> {
    fn push(&mut self, item: T);

    fn pop(&mut self) -> Option<T>;
}

impl<T> Stack<T> for Vec<T> {
    fn push(&mut self, item: T) {
        self.push(item);
    }

    fn pop(&mut self) -> Option<T> {
        self.pop()
    }
}

impl<T> Stack<T> for DiskVec<T>
where
    T: Serialize + DeserializeOwned + Default,
{
    fn push(&mut self, item: T) {
        let _ = DiskVec::push(self, &item);
    }

    fn pop(&mut self) -> Option<T> {
        DiskVec::pop(self).unwrap()
    }
}

#[cfg(test)]
#[allow(unused_variables)]
#[allow(unused_imports)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_stack_vec() {
        let mut stack: Box<dyn Stack<usize>> = Box::<Vec<usize>>::default();

        stack.push(0);
        stack.push(1);
        stack.push(2);

        assert_eq!(stack.pop(), Some(2));
        assert_eq!(stack.pop(), Some(1));
        assert_eq!(stack.pop(), Some(0));
        assert_eq!(stack.pop(), None);
    }

    #[test]
    fn test_queue_disk_vec() {
        let tmp_dir = tempdir().unwrap();
        let capacity = 10;
        let vec = DiskVec::<usize>::new(tmp_dir.path().join("vec.bin"), capacity).unwrap();
        let mut stack: Box<dyn Stack<usize>> = Box::new(vec);

        stack.push(0);
        stack.push(1);
        stack.push(2);

        assert_eq!(stack.pop(), Some(2));
        assert_eq!(stack.pop(), Some(1));
        assert_eq!(stack.pop(), Some(0));
        assert_eq!(stack.pop(), None);
    }
}
