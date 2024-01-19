// Object to track transition progress through CDAWG.
// I prefer an object to a function here because the state is quite complicated.
// This is all 0-indexed because it's independent from building the algorithm.

use graph::indexing::{IndexType, NodeIndex};

#[derive(Debug)]
pub struct CdawgState<Ix>
where
    Ix: IndexType,
{
    pub state: NodeIndex<Ix>,
    pub token: u16,  // First token of active edge.
    pub start: usize,  // Start of active edge.
    pub idx: usize,  // Progress along active edge in [start, end]
    pub end: usize,  // End of active edge.
    pub target: NodeIndex<Ix>,
    pub length: u64,
}
