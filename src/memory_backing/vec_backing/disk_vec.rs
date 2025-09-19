use std::fs::File;
use std::marker;
use std::path::Path;

use anyhow::{bail, Result};
use bincode::Options;
use memmap2::MmapOptions;
use serde::de::DeserializeOwned;
use serde::Serialize;

enum Mmap {
    Mmap(memmap2::Mmap),
    MmapMut(memmap2::MmapMut),
}

/// A vec-like data structure with limited functionality that's backed by a file on disk.
pub struct DiskVec<T>
where
    T: Sized,
{
    item_size: usize,
    capacity: usize,
    len: usize,
    mmap: Mmap,
    file: File,
    _marker: marker::PhantomData<T>,
}

impl<T> DiskVec<T>
where
    T: Serialize + DeserializeOwned + Default,
{
    /// Create a new mutable `DiskVec<T>` with the given file path.
    ///
    /// Fails if the corresponding file already exists.
    pub fn new<P: AsRef<Path> + std::fmt::Debug>(path: P, capacity: usize) -> Result<Self> {
        let item_size = std::mem::size_of::<T>();
        if path.as_ref().is_file() {
            bail!("{path:?} aleady exists!");
        }
        let file = File::options()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&path)?;
        file.set_len((capacity * item_size) as u64)?;
        let mmap = unsafe { MmapOptions::new().map_mut(&file)? };
        Ok(Self {
            item_size,
            capacity,
            len: 0,
            mmap: Mmap::MmapMut(mmap),
            file,
            _marker: marker::PhantomData::<T>,
        })
    }

    /// Load a read-only `DiskVec<T>` from an existing file.
    pub fn load<P: AsRef<Path> + std::fmt::Debug>(path: P) -> Result<Self> {
        let item_size = std::mem::size_of::<T>();
        let file = File::options().read(true).open(&path)?;
        let len = (file.metadata()?.len() as usize) / item_size;
        let mmap = unsafe { MmapOptions::new().map(&file)? };
        Ok(Self {
            item_size,
            capacity: len,
            len,
            mmap: Mmap::Mmap(mmap),
            file,
            _marker: marker::PhantomData::<T>,
        })
    }

    /// Turn a `Vec<T>` into a new `DiskVec<T>`.
    pub fn from_vec<P: AsRef<Path> + std::fmt::Debug>(vec: &Vec<T>, path: P) -> Result<Self> {
        let len = vec.len();
        let mut disk_vec = Self::new(path, len)?;
        for item in vec {
            disk_vec.push(item)?;
        }
        disk_vec.make_read_only()
    }

    /// Convert a writable `DiskVec<T>` into a read-only `DiskVec<T>`.
    pub fn make_read_only(mut self) -> Result<Self> {
        if self.len < self.capacity {
            let new_file_len = self.len * self.item_size;
            self.file.set_len(new_file_len as u64)?;
        }
        if let Mmap::MmapMut(mmap) = self.mmap {
            mmap.flush()?;
        }
        self.file.metadata()?.permissions().set_readonly(true);
        self.mmap = Mmap::Mmap(unsafe { MmapOptions::new().map(&self.file)? });
        Ok(self)
    }

    pub fn try_reserve(&mut self, additional: usize) -> Result<()> {
        let new_capacity = self.capacity + additional;
        self.file.set_len((new_capacity * self.item_size) as u64)?;
        self.mmap = Mmap::MmapMut(unsafe { MmapOptions::new().map_mut(&self.file)? });
        self.capacity = new_capacity;
        Ok(())
    }

    /// Push a new item onto the `DiskVec<T>`.
    pub fn push(&mut self, value: &T) -> Result<()> {
        if self.len == self.capacity {
            self.try_reserve(1)?;
        }
        self._set(self.len, value)?;
        self.len += 1;
        Ok(())
    }

    /// A hacky way to use the DiskVec as a stack.
    /// Possible strange interactions with other methods that use len!!
    pub fn pop(&mut self) -> Result<Option<T>> {
        if self.len == 0 {
            return Ok(None);
        }
        self.len -= 1;
        Ok(Some(self.get(self.len)?))
    }

    fn _set(&mut self, index: usize, value: &T) -> Result<()> {
        if let Mmap::MmapMut(ref mut mmap) = self.mmap {
            let serialized = bincode::DefaultOptions::new()
                .with_fixint_encoding()
                .serialize(value)?;
            if serialized.len() > self.item_size {
                bail!("error inserting value into array, size of serialized item ({}) does not match expected size ({})!", serialized.len(), self.item_size);
            }
            let start_idx = index * self.item_size;
            mmap[start_idx..(start_idx + serialized.len())].copy_from_slice(&serialized[..]);
        } else {
            bail!("this DiskVec is read only!");
        }
        Ok(())
    }

    /// Set the item at the given index.
    pub fn set(&mut self, index: usize, value: &T) -> Result<()> {
        if index > self.len {
            bail!(
                "index {} out of bounds for DiskVec of size {}",
                index,
                self.len
            );
        }
        self._set(index, value)
    }

    /// The number of items in the `DiskVec`.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the `DiskVec` is empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Get the item at the given index.
    pub fn get(&self, index: usize) -> Result<T> {
        if index > self.len {
            bail!(
                "index {} out of bounds for DiskVec of size {}",
                index,
                self.len
            );
        }
        let start_index = index * self.item_size;
        let bytes = match &self.mmap {
            Mmap::Mmap(mmap) => &mmap[start_index..(start_index + self.item_size)],
            Mmap::MmapMut(mmap) => &mmap[start_index..(start_index + self.item_size)],
        };
        let deserialized = bincode::DefaultOptions::new()
            .with_fixint_encoding()
            .allow_trailing_bytes()
            .deserialize::<T>(bytes)?;
        Ok(deserialized)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::avl_graph::AvlNode;
    use crate::graph::indexing::{DefaultIx, NodeIndex};
    use crate::graph::traits::NodeRef;
    use crate::weight::{DefaultWeight, Weight};
    use serde::Deserialize;
    use tempfile::tempdir;

    #[derive(Serialize, Deserialize, Default, Debug)]
    struct Foo {
        x: usize,
        y: usize,
    }

    #[test]
    fn test_disk_vec_push_set_get() {
        let tmp_dir = tempdir().unwrap();

        // Create a new disk vec.
        let mut disk_vec = DiskVec::<Foo>::new(tmp_dir.path().join("vec.bin"), 4).unwrap();
        assert_eq!(disk_vec.len(), 0);

        disk_vec.push(&Foo { x: 17, y: 0 }).unwrap();
        assert_eq!(disk_vec.len(), 1);
        assert_eq!(disk_vec.get(0).unwrap().x, 17);

        disk_vec.push(&Foo { x: 0, y: 1 }).unwrap();
        assert_eq!(disk_vec.len(), 2);
        assert_eq!(disk_vec.get(1).unwrap().x, 0);

        disk_vec.set(1, &Foo { x: 2, y: 1 }).unwrap();
        assert_eq!(disk_vec.len(), 2);
        assert_eq!(disk_vec.get(1).unwrap().x, 2);
    }

    #[test]
    fn test_node_disk_vec_push_set_get() {
        type T = AvlNode<DefaultWeight, DefaultIx>;
        let tmp_dir = tempdir().unwrap();

        let mut disk_vec: DiskVec<T> = DiskVec::new(tmp_dir.path().join("nodes.vec"), 8).unwrap();
        assert_eq!(disk_vec.len(), 0);

        let node: T = AvlNode::new(DefaultWeight::new(32, Some(NodeIndex::new(2)), 2));
        let _ = disk_vec.push(&node);
        assert_eq!(disk_vec.get(0).unwrap().get_length(), 32);

        let new_node: T = AvlNode::new(DefaultWeight::new(42, Some(NodeIndex::new(2)), 2));
        let _ = disk_vec.set(0, &new_node);
        assert_eq!(disk_vec.get(0).unwrap().get_length(), 42);
    }

    #[test]
    fn test_from_vec() {
        let tmp_dir = tempdir().unwrap();

        let vec = vec![Foo { x: 0, y: 1 }, Foo { x: 2, y: 3 }];
        let disk_vec = DiskVec::<Foo>::from_vec(&vec, tmp_dir.path().join("vec.bin")).unwrap();
        assert_eq!(disk_vec.len(), 2);
        assert_eq!(disk_vec.get(1).unwrap().x, 2);
    }
}
