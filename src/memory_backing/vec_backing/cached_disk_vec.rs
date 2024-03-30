use anyhow::Result;
use lru::LruCache;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::num::NonZeroUsize;
use std::path::Path;

use super::DiskVec;
use crate::graph::indexing::{DefaultIx, IndexType};

/// A DiskVec where recently accessed entries are cached in RAM.
pub struct CachedDiskVec<T, Ix = DefaultIx>
where
    T: Sized,
    Ix: IndexType,
{
    vec: DiskVec<T>,
    cache: Option<LruCache<Ix, T>>,
}

impl<T, Ix> CachedDiskVec<T, Ix>
where
    T: Serialize + DeserializeOwned + Default + Copy,
    Ix: IndexType,
{
    /// Create a new mutable `DiskVec<T>` with the given file path.
    ///
    /// Fails if the corresponding file already exists.
    pub fn new<P: AsRef<Path> + std::fmt::Debug>(
        path: P,
        capacity: usize,
        cache_size: usize,
    ) -> Result<Self> {
        let vec = DiskVec::new(path, capacity)?;
        let cache = if cache_size > 0 {
            Some(LruCache::new(NonZeroUsize::new(cache_size).unwrap()))
        } else {
            None
        };
        Ok(Self { vec, cache })
    }

    /// Load a read-only `DiskVec<T>` from an existing file.
    pub fn load<P: AsRef<Path> + std::fmt::Debug>(path: P, cache_size: usize) -> Result<Self> {
        let vec = DiskVec::load(path)?;
        let cache = if cache_size > 0 {
            Some(LruCache::new(NonZeroUsize::new(cache_size).unwrap()))
        } else {
            None
        };
        Ok(Self { vec, cache })
    }

    /// Turn a `Vec<T>` into a new `DiskVec<T>`.
    pub fn from_vec<P: AsRef<Path> + std::fmt::Debug>(
        vec: &Vec<T>,
        path: P,
        cache_size: usize,
    ) -> Result<Self> {
        let vec = DiskVec::from_vec(vec, path)?;
        let cache = if cache_size > 0 {
            Some(LruCache::new(NonZeroUsize::new(cache_size).unwrap()))
        } else {
            None
        };
        Ok(Self { vec, cache })
    }

    // /// Convert a writable `DiskVec<T>` into a read-only `DiskVec<T>`.
    // pub fn make_read_only(mut self) -> Result<()> {
    //     let _ = self.vec.make_read_only()?;
    //     Ok(())
    // }

    pub fn try_reserve(&mut self, additional: usize) -> Result<()> {
        self.vec.try_reserve(additional)
    }

    /// Push a new item onto the `DiskVec<T>`.
    pub fn push(&mut self, value: &T) -> Result<()> {
        self.vec.push(value)?;
        Ok(())
    }

    fn pop_cache(&mut self, index: usize) {
        // See https://users.rust-lang.org/t/if-let-passing-a-mutable-ref-option-not-working/31300
        if let Some(cache) = self.cache.as_mut() {
            cache.pop(&Ix::new(index));
        }
    }

    /// A hacky way to use the DiskVec as a stack.
    /// Possible strange interactions with other methods that use len!!
    pub fn pop(&mut self) -> Result<Option<T>> {
        let value = self.vec.pop()?;
        if value.is_some() {
            self.pop_cache(self.vec.len());
        }
        Ok(value)
    }

    /// Set the item at the given index. Removes that item from the cache.
    pub fn set(&mut self, index: usize, value: &T) -> Result<()> {
        self.vec.set(index, value)?;
        self.pop_cache(index);
        Ok(())
    }

    /// The number of items in the `DiskVec`.
    pub fn len(&self) -> usize {
        self.vec.len()
    }

    /// Returns `true` if the `DiskVec` is empty.
    pub fn is_empty(&self) -> bool {
        self.vec.is_empty()
    }

    /// Get the item at the given index.
    pub fn get(&mut self, index: usize) -> Result<T> {
        let idx = Ix::new(index);
        if let Some(cache) = self.cache.as_mut() {
            match cache.get(&idx) {
                Some(value) => Ok(*value),
                None => {
                    let value = self.vec.get(index)?;
                    cache.put(idx, value);
                    Ok(value)
                }
            }
        } else {
            self.vec.get(index)
        }
    }

    /// Get number of elements currently in the cache.
    pub fn get_cache_len(&self) -> usize {
        match self.cache.as_ref() {
            Some(cache) => cache.len(),
            None => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_push_get_integers() {
        let tmp_dir = tempdir().unwrap();
        let capacity = 10;
        let cache_size = 5;
        let mut vec: CachedDiskVec<usize> =
            CachedDiskVec::new(tmp_dir.path().join("vec.bin"), capacity, cache_size).unwrap();
        assert_eq!(vec.len(), 0);

        for idx in 0..10 {
            let value = idx + 10;
            let _ = vec.push(&value);
        }

        // Before "getting", cache is fully empty.
        for idx in 0..10 {
            assert_eq!(vec.cache.as_mut().unwrap().get(&DefaultIx::new(idx)), None);
        }

        // Check that "get" returns the right values.
        for idx in 0..10 {
            assert_eq!(vec.get(idx).unwrap(), idx + 10);
        }

        // Only the last five things should fit in the cache.
        for idx in 0..5 {
            assert_eq!(vec.cache.as_mut().unwrap().get(&DefaultIx::new(idx)), None);
        }
        for idx in 5..10 {
            let value = idx + 10;
            assert_eq!(
                vec.cache.as_mut().unwrap().get(&DefaultIx::new(idx)),
                Some(&value)
            );
        }
    }

    #[test]
    fn test_push_set_get_integers() {
        let tmp_dir = tempdir().unwrap();
        let capacity = 10;
        let cache_size = 5;
        let mut vec: CachedDiskVec<usize> =
            CachedDiskVec::new(tmp_dir.path().join("vec.bin"), capacity, cache_size).unwrap();
        assert_eq!(vec.len(), 0);

        for idx in 0..10 {
            let value = idx + 10;
            let _ = vec.push(&value);
        }

        for idx in 0..5 {
            let value = idx + 20;
            let _ = vec.set(idx, &value);
        }

        for idx in 0..10 {
            assert_eq!(vec.cache.as_mut().unwrap().get(&DefaultIx::new(idx)), None);
        }

        for idx in 0..5 {
            assert_eq!(vec.get(idx).unwrap(), idx + 20);
        }
        for idx in 5..10 {
            assert_eq!(vec.get(idx).unwrap(), idx + 10);
        }
    }

    #[test]
    fn test_empty_cache() {
        let tmp_dir = tempdir().unwrap();
        let capacity = 10;
        let vec: CachedDiskVec<usize> =
            CachedDiskVec::new(tmp_dir.path().join("vec.bin"), capacity, 0).unwrap();
        assert!(vec.cache.is_none());
    }
}
