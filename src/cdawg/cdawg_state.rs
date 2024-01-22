// Object to track transition progress through CDAWG.
// I prefer an object to a function here because the state is quite complicated.
// This is all 0-indexed because it's independent from building the algorithm.

use graph::indexing::{IndexType, NodeIndex};

#[derive(Debug, Copy, Clone)]
pub struct CdawgState<Ix>
where
    Ix: IndexType,
{
    pub state: NodeIndex<Ix>,  // Original state of active edge.
    pub edge_start: usize,  // Original start of active edge.
    pub start: usize,  // Start of gamma.
    pub end: usize,  // End of gamma.
    pub target: Option<NodeIndex<Ix>>,  // Target of active edge.
    pub length: u64,  // Current length.
}