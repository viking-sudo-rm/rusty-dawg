use serde::{Deserialize,Serialize};
use std::cmp::{Eq, Ord, PartialEq, PartialOrd, Ordering};

use graph::indexing::{DefaultIx, IndexType};

// TODO: Can simply remove this type and use (Ix, Ix)

#[derive(Eq, Ord, Copy, Clone, Default, Debug, Deserialize, Serialize)]
pub struct CdawgEdgeWeight<Ix: IndexType = DefaultIx>
{
    #[serde(bound(
        serialize = "Ix: Serialize",
        deserialize = "Ix: Deserialize<'de>",
    ))]
    pub start: Ix,  
    pub end: Ix,
}

impl<Ix> CdawgEdgeWeight<Ix>
where
    Ix: IndexType,
{
    pub fn new(start: usize, end: usize) -> Self {
        Self {start: Ix::new(start), end: Ix::new(end)}
    }

    pub fn get_span(&self) -> (usize, usize) {
        (self.start.index(), self.end.index())
    }
}

// These traits are not used, but just added here for compatability.
// TODO: Remove need for them.

impl<Ix> PartialEq for CdawgEdgeWeight<Ix>
where
    Ix: IndexType,
{
    fn eq(&self, other: &Self) -> bool {
        true  // FAKE
    }
}

impl<Ix> PartialOrd for CdawgEdgeWeight<Ix>
where
    Ix: IndexType,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(Ordering::Equal)  // FAKE
    }
}