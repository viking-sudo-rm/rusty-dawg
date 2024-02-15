// Implement the VecBacking interface for DiskVec.

use super::disk_mut_refs::{DiskVecItem, MutRef};
use crate::memory_backing::{CachedDiskVec, VecBacking};
use anyhow::Result;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

pub struct Vec<T>
where
    T: Sized,
{
    disk_vec: Rc<RefCell<CachedDiskVec<T>>>,
}

impl<T> Vec<T>
where
    T: DiskVecItem + Default + Serialize + DeserializeOwned + Copy,
{
    pub fn new<P: AsRef<Path> + std::fmt::Debug>(
        path: P,
        capacity: usize,
        cache_size: usize,
    ) -> Result<Self> {
        let disk_vec = CachedDiskVec::new(path, capacity, cache_size)?;
        Ok(Self {
            disk_vec: Rc::new(RefCell::new(disk_vec)),
        })
    }

    pub fn load<P: AsRef<Path> + std::fmt::Debug>(path: P, cache_size: usize) -> Result<Self> {
        let disk_vec = CachedDiskVec::load(path, cache_size)?;
        Ok(Self {
            disk_vec: Rc::new(RefCell::new(disk_vec)),
        })
    }
}

impl<T> VecBacking<T> for Vec<T>
where
    T: DiskVecItem + Default + Serialize + DeserializeOwned + Copy,
{
    type TRef = T;
    type TMutRef = T::MutRef;

    fn len(&self) -> usize {
        self.disk_vec.borrow().len()
    }

    fn push(&mut self, item: T) {
        let _ = self.disk_vec.borrow_mut().push(&item);
    }

    fn index(&self, index: usize) -> T {
        self.disk_vec.borrow_mut().get(index).unwrap()
    }

    fn index_mut(&mut self, index: usize) -> T::MutRef {
        T::MutRef::new(self.disk_vec.clone(), index)
    }
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use super::*;
    use crate::memory_backing::disk_backing::disk_mut_refs::MutRef;
    use crate::memory_backing::VecBacking;
    use serde::Deserialize;
    use std::cell::RefCell;
    use std::rc::Rc;
    use tempfile::tempdir;

    pub struct DummyMutRef {
        disk_vec: Rc<RefCell<CachedDiskVec<u8>>>,
        index: usize,
    }

    impl MutRef<u8> for DummyMutRef {
        fn new(disk_vec: Rc<RefCell<CachedDiskVec<u8>>>, index: usize) -> Self {
            Self { disk_vec, index }
        }
    }

    impl DummyMutRef {
        pub fn meaning_of_life(self) {
            let mut disk_vec = self.disk_vec.borrow_mut();
            let _ = disk_vec.set(self.index, &42);
        }
    }

    impl DiskVecItem for u8 {
        type MutRef = DummyMutRef;
    }

    #[test]
    fn test_diskvec_as_veclike() {
        let tmp_dir = tempdir().unwrap();
        let disk_vec = Vec::<u8>::new(tmp_dir.path().join("vec.bin"), 4, 0).unwrap();
        let mut mb: Box<dyn VecBacking<u8, TRef = u8, TMutRef = DummyMutRef>> = Box::new(disk_vec);

        mb.push(20);
        assert_eq!(mb.index(0), 20);
        mb.index_mut(0).meaning_of_life();
        assert_eq!(mb.index(0), 42);
    }
}
