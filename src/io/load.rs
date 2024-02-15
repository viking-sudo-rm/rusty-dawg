use crate::dawg::Dawg;
use crate::graph::indexing::DefaultIx;
use crate::weight::Weight;
use bincode::deserialize_from;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::cmp::Eq;
use std::error::Error;
use std::fmt::Debug;
use std::fs;

use crate::memory_backing::{CacheConfig, DiskBacking};

pub trait Load {
    fn load(load_path: &str, cache_config: CacheConfig) -> Result<Self, Box<dyn Error>>
    where
        Self: Sized;
}

// load_path should be a file with a serialized Dawg object.
impl<E, W> Load for Dawg<E, W>
where
    E: Eq + Copy + Debug + for<'de> Deserialize<'de>,
    W: Weight + Copy + Serialize + for<'a> Deserialize<'a> + Clone,
{
    fn load(load_path: &str, _cache_config: CacheConfig) -> Result<Self, Box<dyn Error>> {
        let file = fs::OpenOptions::new().read(true).open(load_path)?;
        Ok(deserialize_from(&file)?)
    }
}

// load_path should be a directory containing two nodes.vec and edges.vec.
impl<E, W> Load for Dawg<E, W, DefaultIx, DiskBacking<W, E, DefaultIx>>
where
    E: Eq + Copy + Ord + Debug + Serialize + DeserializeOwned + Default,
    W: Weight + Copy + Clone + Serialize + DeserializeOwned + Default,
{
    fn load(load_path: &str, cache_config: CacheConfig) -> Result<Self, Box<dyn Error>> {
        let dawg = Dawg::load(load_path, cache_config)?;
        Ok(dawg)
    }
}
