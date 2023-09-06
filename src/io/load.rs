use serde::{Serialize, Deserialize};
use std::error::Error;
use bincode::deserialize_from;
use std::fs;
use std::cmp::Eq;
use std::fmt::Debug;
use weight::Weight;
use dawg::Dawg;


pub trait Load {

    fn load(load_path: &str) -> Result<Self, Box<dyn Error>> where Self: Sized;

}

impl<E, W> Load for Dawg<E, W>
where
    E: Eq + Copy + Debug + for<'de> Deserialize<'de>,
    W: Weight + Serialize + for<'a> Deserialize<'a> + Clone,
{

    fn load(load_path: &str) -> Result<Self, Box<dyn Error>> {
        let file = fs::OpenOptions::new().read(true).open(&load_path)?;
        Ok(deserialize_from(&file)?)
    }

}
