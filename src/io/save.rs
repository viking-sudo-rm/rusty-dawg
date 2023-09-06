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

// Disk-backed implementation can close the file that is open and not use save_path (or assert it matches).
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