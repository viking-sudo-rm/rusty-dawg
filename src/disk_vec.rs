use std::convert::TryInto;
use std::fs::{self, File};
use std::io::prelude::*;
use std::io::SeekFrom;
use std::marker;
use std::ops::Index;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{anyhow, bail, Result};
use bincode::Options;
use fslock::LockFile;
use serde::de::{Deserialize, DeserializeOwned};
use serde::Serialize;
use tempfile::NamedTempFile;

/// A vec-like data structure with limited functionality that's backed by a file on disk.
/// This can only be used with types that always serialize to the same number of bytes.
pub struct DiskVec<T>
where
    T: Serialize + DeserializeOwned + Default,
{
    path: PathBuf,
    file: Mutex<File>,
    lockfile: LockFile,
    len: usize,
    item_size: usize,
    buffer: Mutex<Vec<u8>>,
    read_only: bool,
    _marker: marker::PhantomData<T>,
}

impl<T> DiskVec<T>
where
    T: Serialize + DeserializeOwned + Default,
{
    /// Create a new `DiskVec<T>` with the given file path.
    ///
    /// Fails if the corresponding file already exists.
    pub fn new<P: AsRef<Path> + std::fmt::Debug>(path: P) -> Result<Self> {
        if path.as_ref().is_file() {
            bail!("{path:?} aleady exists!");
        }
        let file = File::options()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)?;
        Self::from_file_path(file, path, false)
    }

    /// Load a read-only `DiskVec<T>` from an existing file.
    pub fn load<P: AsRef<Path> + std::fmt::Debug>(path: P) -> Result<Self> {
        let file = File::options().read(true).open(&path)?;
        Self::from_file_path(file, path, true)
    }

    fn from_file_path<P: AsRef<Path> + std::fmt::Debug>(
        file: File,
        path: P,
        read_only: bool,
    ) -> Result<Self> {
        // Try to acquire the lock file.
        let lock_path = path.as_ref().with_extension("lock");
        let mut lockfile = LockFile::open(&lock_path)?;
        let acquire_start = Instant::now();
        while !lockfile.try_lock()? {
            if acquire_start.elapsed().as_secs() > 1 {
                bail!("could not acquire lockfile for {:?}.\n\
                      This means there is another process holding a write lock on the file, or a previous process didn't clean up its lock properly.\n\
                      In the latter case, you could just delete the lock file at {:?}", path, lock_path);
            } else {
                thread::sleep(Duration::from_millis(10));
            }
        }

        // If we're opening with read only we can unlock the lock file right away to allow
        // other readers.
        if read_only {
            lockfile.unlock()?;
        }

        // Get size of serialized items and initialize buffer.
        let item_size = Self::get_item_size()?;
        let buffer = vec![0; item_size];

        // Get size of file to determine number of items.
        let size_in_bytes = file.metadata()?.len() as usize;

        Ok(Self {
            path: path.as_ref().into(),
            file: Mutex::new(file),
            lockfile,
            len: size_in_bytes / item_size,
            item_size,
            buffer: Mutex::new(buffer),
            read_only,
            _marker: marker::PhantomData,
        })
    }

    /// Turn a `Vec<T>` into a new `DiskVec<T>`.
    pub fn from_vec<P: AsRef<Path> + std::fmt::Debug>(vec: Vec<T>, path: P) -> Result<Self> {
        if path.as_ref().is_file() {
            bail!("{path:?} aleady exists!");
        }

        // Get a temporary file to initialize the new `DiskVec<T>` with.
        let parent_dir = path
            .as_ref()
            .parent()
            .ok_or_else(|| anyhow!("{:?} does not have a parent!", path))?;
        let tmp_file = NamedTempFile::new_in(parent_dir)?;
        let file = File::options()
            .read(true)
            .write(true)
            .create(true)
            .open(&tmp_file)?;

        // Initialize and populate the new `DiskVec<T>`.
        let mut disk_vec = Self::from_file_path(file, &tmp_file, false)?;
        for value in vec {
            disk_vec.push(value)?;
        }

        // Move to the target path.
        disk_vec.rename_to(path.as_ref())
    }

    /// Rename the underlying file to another path on the same filesystem, consuming the current `DiskVec<T>`
    /// and returning a new read-only version.
    pub fn rename_to<P: AsRef<Path> + std::fmt::Debug>(self, to: P) -> Result<Self> {
        if to.as_ref().is_file() {
            bail!("{:?} aleady exists!", to);
        }

        fs::rename(self.path, &to)?;
        Self::load(to)
    }

    /// Convert a writable `DiskVec<T>` into a read-only `DiskVec<T>`, allowing other read-only `DiskVec`s to
    /// exist with the same backing file.
    pub fn read_only(&mut self) -> Result<()> {
        if !self.read_only {
            self.lockfile.unlock()?;
            self.read_only = true;
        }
        Ok(())
    }

    /// Push a new item onto the `DiskVec<T>`.
    pub fn push(&mut self, value: T) -> Result<()> {
        if self.read_only {
            bail!("this DiskVec is read only!");
        }

        // Serialize item.
        let encoded = bincode::DefaultOptions::new()
            .with_fixint_encoding()
            .serialize(&value)?;
        if encoded.len() != self.item_size {
            bail!("error inserting value into array, size of serialized item ({}) does not match expected size ({})!", encoded.len(), self.item_size);
        }

        // Get lock on file.
        let mut file = self
            .file
            .lock()
            .map_err(|_| anyhow!("failed to acquire inner mutex on file"))?;

        // Write serialized item to file.
        (*file).seek(SeekFrom::Start(
            (self.item_size * self.len).try_into().unwrap(),
        ))?;
        (*file).write_all(&encoded)?;
        (*file).sync_all()?;

        self.len += 1;

        Ok(())
    }

    /// Set the item at the given index.
    pub fn set(&mut self, index: usize, value: T) -> Result<()> {
        if self.read_only {
            bail!("this DiskVec is read only!");
        }
        if index > self.len {
            bail!("index out of bounds");
        }

        let encoded = bincode::DefaultOptions::new()
            .with_fixint_encoding()
            .serialize(&value)?;
        if encoded.len() != self.item_size {
            bail!("error inserting value into array, size of serialized item ({}) does not match expected size ({})!", encoded.len(), self.item_size);
        }

        let mut file = self
            .file
            .lock()
            .map_err(|_| anyhow!("failed to acquire inner mutex on file"))?;
        (*file).seek(SeekFrom::Start(
            (self.item_size * index).try_into().unwrap(),
        ))?;
        (*file).write_all(&encoded)?;
        (*file).sync_all()?;

        Ok(())
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
        if index >= self.len {
            bail!("index out of bounds");
        }

        // Lock file and buffer.
        let mut file = self
            .file
            .lock()
            .map_err(|_| anyhow!("failed to acquire inner mutex on file"))?;
        let mut buffer = self
            .buffer
            .lock()
            .map_err(|_| anyhow!("failed to acquire inner mutex on buffer"))?;

        // Read from file and deserialize item.
        (*file).seek(SeekFrom::Start((self.item_size * index).try_into()?))?;
        (*file).read_exact(&mut buffer)?;
        let value = bincode::DefaultOptions::new()
            .with_fixint_encoding()
            .deserialize::<T>(&buffer)?;

        Ok(value)
    }

    /// Get the size of a serialized item in bytes.
    fn get_item_size() -> Result<usize> {
        let tmp_item = T::default();
        let size = bincode::DefaultOptions::new()
            .with_fixint_encoding()
            .serialized_size(&tmp_item)?;
        Ok(size as usize)
    }
}

impl<T> Index<usize> for DiskVec<T>
where T: Serialize + DeserializeOwned + Default
{
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.get(index).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
        let mut disk_vec = DiskVec::<Foo>::new(tmp_dir.path().join("vec.bin")).unwrap();
        assert_eq!(disk_vec.len(), 0);

        // Trying to load another DiskVec on the same file right now should result in an error
        // since we won't be able to acquire the lock file.
        assert!(DiskVec::<Foo>::load(tmp_dir.path().join("vec.bin")).is_err());

        disk_vec.push(Foo { x: 17, y: 0 }).unwrap();
        assert_eq!(disk_vec.len(), 1);
        assert_eq!(disk_vec.get(0).unwrap().x, 17);

        disk_vec.push(Foo { x: 0, y: 1 }).unwrap();
        assert_eq!(disk_vec.len(), 2);
        assert_eq!(disk_vec.get(1).unwrap().x, 0);

        disk_vec.set(1, Foo { x: 2, y: 1 }).unwrap();
        assert_eq!(disk_vec.len(), 2);
        assert_eq!(disk_vec.get(1).unwrap().x, 2);
    }

    #[test]
    fn test_disk_vec_locking() {
        let tmp_dir = tempdir().unwrap();

        // Create a new disk vec.
        {
            let mut disk_vec = DiskVec::<Foo>::new(tmp_dir.path().join("vec.bin")).unwrap();
            assert_eq!(disk_vec.len(), 0);

            // Trying to load another DiskVec on the same file right now should result in an error
            // since we won't be able to acquire the lock file.
            assert!(DiskVec::<Foo>::load(tmp_dir.path().join("vec.bin")).is_err());

            disk_vec.push(Foo { x: 17, y: 0 }).unwrap();
            assert_eq!(disk_vec.len(), 1);
            assert_eq!(disk_vec.get(0).unwrap().x, 17);
        }

        // Load a DiskVec from the existing file. Now that the other DiskVec has been dropped we
        // should be able to acquire the lock file.
        let disk_vec = DiskVec::<Foo>::load(tmp_dir.path().join("vec.bin")).unwrap();
        assert_eq!(disk_vec.len(), 1);
        assert_eq!(disk_vec.get(0).unwrap().x, 17);
    }

    #[test]
    fn test_disk_vec_unlocking() {
        let tmp_dir = tempdir().unwrap();

        // Create a new disk vec.
        let mut disk_vec = DiskVec::<Foo>::new(tmp_dir.path().join("vec.bin")).unwrap();
        assert_eq!(disk_vec.len(), 0);

        disk_vec.push(Foo { x: 17, y: 0 }).unwrap();
        assert_eq!(disk_vec.len(), 1);
        assert_eq!(disk_vec.get(0).unwrap().x, 17);

        // Setting the current DiskVec to read-only will allow other DiskVecs to exist with the
        // same backing file.
        disk_vec.read_only().unwrap();
        let disk_vec2 = DiskVec::<Foo>::load(tmp_dir.path().join("vec.bin")).unwrap();
        assert_eq!(disk_vec2.len(), 1);
    }

    #[test]
    fn test_from_vec() {
        let tmp_dir = tempdir().unwrap();

        let vec = vec![Foo { x: 0, y: 1 }, Foo { x: 2, y: 3 }];
        let disk_vec = DiskVec::<Foo>::from_vec(vec, tmp_dir.path().join("vec.bin")).unwrap();
        assert_eq!(disk_vec.len(), 2);
        assert_eq!(disk_vec.get(1).unwrap().x, 2);
    }
}
