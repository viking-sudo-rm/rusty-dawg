// Implement the VecBacking interface for DiskVec.

use graph::memory_backing::vec_backing::VecBacking;
use graph::avl_graph::node::Node;
use graph::avl_graph::edge::Edge;
use graph::memory_backing::disk_backing::disk_vec::DiskVec;
use graph::memory_backing::disk_backing::disk_mut_refs::{DiskNodeMutRef, DiskEdgeMutRef, DiskVecItem};
use serde::Serialize;
use serde::de::DeserializeOwned;

impl<T> VecBacking<T, T, T::MutRef<'_>> for DiskVec<T>
where
    T: DiskVecItem + Default + Serialize + DeserializeOwned,
{
    fn len(&self) -> usize {
        DiskVec::len(self)
    }

    fn push(&mut self, item: T) {
        DiskVec::push(self, &item);
    }

    fn index(&self, index: usize) -> T {
        self.get(index).unwrap()
    }

    fn index_mut(&mut self, index: usize) -> T::MutRef<'l> {
        T::new_mut_ref(self, index)
    }
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use tempfile::tempdir;
    use graph::memory_backing::vec_backing::VecBacking;

    pub struct DummyMutRef<'a> {
        disk_vec: &'a mut DiskVec<u8>,
        index: usize,
    }

    impl<'a> DummyMutRef<'a> {
        pub fn meaning_of_life(self) {
            let _ = self.disk_vec.set(self.index, &42);
        }
    }

    impl DiskVecItem for u8 {
        type MutRef<'a> = DummyMutRef<'a>;

        fn new_mut_ref<'a>(disk_vec: &'a mut DiskVec<u8>, index: usize) -> Self::MutRef<'a> {
            DummyMutRef {disk_vec, index}
        }
    }

    #[test]
    fn test_diskvec_as_veclike() {
        let tmp_dir = tempdir().unwrap();
        let mut disk_vec = DiskVec::<u8>::new(tmp_dir.path().join("vec.bin"), 4).unwrap();
        let mut mb: Box<dyn VecBacking<u8, u8, DummyMutRef>> = Box::new(disk_vec);

        mb.push(20);
        assert_eq!(mb.index(0), 12);
        mb.index_mut(0).meaning_of_life();
        assert_eq!(mb.index(0), 42);
    }
}
