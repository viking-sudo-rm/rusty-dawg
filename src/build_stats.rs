// Stats for logging during the building of a DAWG or CDAWG.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::Path;

use crate::cdawg::Cdawg;
use crate::graph::indexing::IndexType;
use crate::memory_backing::MemoryBacking;
use crate::weight::Weight;

#[derive(Serialize, Deserialize)]
pub struct BuildStats {
    pub n_tokens: usize,
    pub n_nodes: usize,
    pub n_edges: usize,
    pub n_bytes: u64,
    pub balance_ratio: f64,
    pub elapsed_time: f32,
}

impl BuildStats {
    pub fn from_cdawg<N, Ix, Mb>(
        cdawg: &Cdawg<N, Ix, Mb>,
        n_tokens: usize,
        n_bytes: u64,
        elapsed_time: f32,
    ) -> Self
    where
        N: Weight + Serialize + for<'de> Deserialize<'de> + Clone + Copy,
        Ix: IndexType,
        Mb: MemoryBacking<N, (Ix, Ix), Ix>,
    {
        Self {
            n_tokens,
            n_nodes: cdawg.node_count(),
            n_edges: cdawg.edge_count(),
            n_bytes,
            balance_ratio: cdawg.balance_ratio(1),
            elapsed_time,
        }
    }

    pub fn get_nodes_per_token(&self) -> f64 {
        (self.n_nodes as f64) / (self.n_tokens as f64)
    }

    pub fn get_edges_per_token(&self) -> f64 {
        (self.n_edges as f64) / (self.n_tokens as f64)
    }

    pub fn get_tokens_per_byte(&self) -> f64 {
        (self.n_tokens as f64) / (self.n_bytes as f64)
    }

    pub fn append_to_jsonl<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let blob = serde_json::to_string(self)?;

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .unwrap();

        Ok(writeln!(file, "{}", blob)?)
    }
}
