use crate::cdawg::Cdawg;
use crate::dawg::Dawg;
use crate::graph::indexing::DefaultIx;
use crate::memory_backing::{DiskBacking, RamBacking};
use crate::weight::Weight;
use serde::de::DeserializeOwned;
use std::error::Error;
use std::fs;

use serde::{Deserialize, Serialize};
use std::cmp::Eq;
use std::fmt::Debug;

use bincode::serialize_into;

pub trait Save {
    fn save(&self, save_path: &str) -> Result<(), Box<dyn Error>>;
}

impl<E, W> Save for Dawg<E, W, DefaultIx, RamBacking<W, E, DefaultIx>>
where
    E: Eq + Copy + Debug + Serialize,
    W: Weight + Serialize + for<'de> Deserialize<'de> + Clone,
{
    fn save(&self, save_path: &str) -> Result<(), Box<dyn Error>> {
        let save_file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(save_path)?;
        serialize_into(&save_file, &self)?;
        Ok(())
    }
}

impl<E, W> Save for Dawg<E, W, DefaultIx, DiskBacking<W, E, DefaultIx>>
where
    E: Eq + Copy + Debug + Serialize + DeserializeOwned + Default,
    W: Weight + Copy + Clone + Serialize + DeserializeOwned + Default,
{
    fn save(&self, _save_path: &str) -> Result<(), Box<dyn Error>> {
        // Everything is already saved with DiskBacking!
        Ok(())
    }
}

impl<W> Save for Cdawg<W, DefaultIx, DiskBacking<W, (DefaultIx, DefaultIx), DefaultIx>>
where
    W: Weight + Copy + Serialize + for<'de> Deserialize<'de> + Clone + Default,
    (DefaultIx, DefaultIx): Serialize + for<'de> Deserialize<'de>,
{
    fn save(&self, save_path: &str) -> Result<(), Box<dyn Error>> {
        Ok(Cdawg::save_metadata(self, save_path)?)
    }
}

impl<W> Save for Cdawg<W, DefaultIx, RamBacking<W, (DefaultIx, DefaultIx), DefaultIx>>
where
    W: Weight + Serialize + for<'de> Deserialize<'de> + Clone + Default,
    (DefaultIx, DefaultIx): Serialize + for<'de> Deserialize<'de>,
{
    fn save(&self, save_path: &str) -> Result<(), Box<dyn Error>> {
        // unimplemented!("Can't yet save CDAWGs on RAM");
        println!("Saving RAM -> disk...");
        // First save whatever is in RAM to disk.
        self.get_graph().save_to_disk(save_path)?;
        // Then generate metadata as we would normally.
        Cdawg::save_metadata(self, save_path)?;
        Ok(())
    }
}
