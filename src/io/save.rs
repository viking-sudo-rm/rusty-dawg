use dawg::Dawg;
use std::error::Error;
use std::fs;
use weight::Weight;

use serde::{Deserialize, Serialize};
use std::cmp::Eq;
use std::fmt::Debug;

use bincode::serialize_into;

pub trait Save {

    fn save(&self, save_path: &str) -> Result<(), Box<dyn Error>>;

}

// TODO: Provide an alternative implementation for a disk-backed version that simply closes the file.
impl<E, W> Save for Dawg<E, W>
where
    E: Eq + Copy + Debug + Serialize,
    W: Weight + Serialize + for<'a> Deserialize<'a> + Clone,
{
    fn save(&self, save_path: &str) -> Result<(), Box<dyn Error>> {
        let save_file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(save_path)?;
        serialize_into(&save_file, &self)?;
        Ok(())
    }

}