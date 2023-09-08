use bincode::deserialize_from;
use dawg::Dawg;
use serde::{Deserialize, Serialize};
use std::cmp::Eq;
use std::error::Error;
use std::fmt::Debug;
use std::fs;
use weight::Weight;

pub trait Load {
    fn load(load_path: &str) -> Result<Self, Box<dyn Error>>
    where
        Self: Sized;
}

// Disk-backed implementation can open the file at load_path.
impl<E, W> Load for Dawg<E, W>
where
    E: Eq + Copy + Debug + for<'de> Deserialize<'de>,
    W: Weight + Serialize + for<'a> Deserialize<'a> + Clone,
{
    fn load(load_path: &str) -> Result<Self, Box<dyn Error>> {
        let file = fs::OpenOptions::new().read(true).open(load_path)?;
        Ok(deserialize_from(&file)?)
    }
}
