use serde::{Deserialize, Serialize};

use crate::graph::indexing::{DefaultIx, IndexType};

// TODO: Can simply remove this type and use (Ix, Ix)

#[derive(Copy, Clone, Default, Debug, Deserialize, Serialize)]
pub struct CdawgEdgeWeight<Ix: IndexType = DefaultIx> {
    #[serde(bound(serialize = "Ix: Serialize", deserialize = "Ix: Deserialize<'de>",))]
    pub start: Ix,
    pub end: Ix,
}

impl<Ix> CdawgEdgeWeight<Ix>
where
    Ix: IndexType,
{
    pub fn new(start: usize, end: usize) -> Self {
        Self {
            start: Ix::new(start),
            end: Ix::new(end),
        }
    }

    pub fn max_value() -> Self {
        Self {
            start: Ix::max_value(),
            end: Ix::max_value(),
        }
    }

    pub fn get_span(&self) -> (usize, usize) {
        (self.start.index(), self.end.index())
    }
}
