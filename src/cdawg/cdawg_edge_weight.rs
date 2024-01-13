use std::cmp::{Eq, Ord, PartialEq, PartialOrd, Ordering};

use graph::indexing::{DefaultIx, IndexType};

#[derive(Eq, Ord, Copy, Clone, Default, Debug)]
pub struct CdawgEdgeWeight<Ix = DefaultIx>
where
    Ix: IndexType,
{
    // Can remove token and just look it up dynamically as tokens[start], but LCDAWG needs it.
    pub token: u16,
    pub start: Ix,  
    pub end: Ix,
}

impl<Ix> CdawgEdgeWeight<Ix>
where
    Ix: IndexType,
{
    // Create a new CdawgEdgeWeight with a token but no span info.
    // We use this for searching for an edge that has a certain first token.
    pub fn new_key(token: u16) -> Self {
        Self {token, start: Ix::new(0), end: Ix::new(0)}
    }

    pub fn new(token: u16, start: usize, end: usize) -> Self {
        Self {token, start: Ix::new(start), end: Ix::new(end)}
    }

    pub fn get_span(&self) -> (usize, usize) {
        (self.start.index(), self.end.index())
    }
}

// Can we make custom PartialEq/PartialOrd objects with a pointer to tokens and pass these when we
// search? Would be much more space-efficient. In this case, we could potentially get rid of this
// object and just use a (start, end) tuple (Span).
//
// However, it seems like the LCDAWG extension *requires* the extra token field, so this is probably
// not worth doing.

impl<Ix> PartialEq for CdawgEdgeWeight<Ix>
where
    Ix: IndexType,
{
    fn eq(&self, other: &Self) -> bool {
        self.token == other.token
    }
}

impl<Ix> PartialOrd for CdawgEdgeWeight<Ix>
where
    Ix: IndexType,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.token.cmp(&other.token))
    }
}