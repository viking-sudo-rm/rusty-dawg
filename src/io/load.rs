use bincode::deserialize_from;
use crate::dawg::Dawg;
use crate::graph::indexing::DefaultIx;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::cmp::Eq;
use std::error::Error;
use std::fmt::Debug;
use std::fs;
use crate::weight::Weight;

use crate::graph::memory_backing::DiskBacking;

pub trait Load {
    fn load(load_path: &str) -> Result<Self, Box<dyn Error>>
    where
        Self: Sized;
}

// load_path should be a file with a serialized Dawg object.
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

// load_path should be a directory containing two nodes.vec and edges.vec.
impl<E, W> Load for Dawg<E, W, DefaultIx, DiskBacking<W, E, DefaultIx>>
where
    E: Eq + Copy + Ord + Debug + Serialize + DeserializeOwned + Default,
    W: Weight + Clone + Serialize + DeserializeOwned + Default,
{
    fn load(load_path: &str) -> Result<Self, Box<dyn Error>> {
        let dawg = Dawg::load(load_path)?;
        Ok(dawg)
    }
}
