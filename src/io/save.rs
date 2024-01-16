use dawg::Dawg;
use graph::indexing::DefaultIx;
use graph::memory_backing::{DiskBacking, RamBacking};
use serde::de::DeserializeOwned;
use std::error::Error;
use std::fs;
use weight::Weight;
use cdawg::Cdawg;
use cdawg::cdawg_edge_weight::CdawgEdgeWeight;

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
            .open(save_path)?;
        serialize_into(&save_file, &self)?;
        Ok(())
    }
}

impl<E, W> Save for Dawg<E, W, DefaultIx, DiskBacking<W, E, DefaultIx>>
where
    E: Eq + Copy + Debug + Serialize + DeserializeOwned + Default,
    W: Weight + Clone + Serialize + DeserializeOwned + Default,
{
    fn save(&self, _save_path: &str) -> Result<(), Box<dyn Error>> {
        // Everything is already saved with DiskBacking!
        Ok(())
    }
}

impl<W> Save for Cdawg<W, DefaultIx, DiskBacking<W, CdawgEdgeWeight<DefaultIx>, DefaultIx>>
where
    W: Weight + Serialize + for<'de> Deserialize<'de> + Clone + Default,
    CdawgEdgeWeight<DefaultIx>: Serialize + for<'de> Deserialize<'de>,
{
    fn save(&self, save_path: &str) -> Result<(), Box<dyn Error>> {
        Ok(self.save(save_path)?)
    }
}

impl<W> Save for Cdawg<W, DefaultIx, RamBacking<W, CdawgEdgeWeight<DefaultIx>, DefaultIx>>
where
    W: Weight + Serialize + for<'de> Deserialize<'de> + Clone + Default,
    CdawgEdgeWeight<DefaultIx>: Serialize + for<'de> Deserialize<'de>,
{
    fn save(&self, save_path: &str) -> Result<(), Box<dyn Error>> {
        unimplemented!("Can't yet save CDAWGs on RAM");
        Ok(())  // TODO
    }
}