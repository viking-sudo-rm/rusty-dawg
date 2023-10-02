use std::cmp::{Eq, Ord, PartialEq, PartialOrd, Ordering};

#[derive(Eq, Ord, Copy, Clone, Default, Debug)]
pub struct CdawgEdgeWeight {
    token: u16,
    start: usize,  // TODO: Can make this lower precision.
    end: usize,  // TODO: Can make this lower precision.
}

impl CdawgEdgeWeight {
    // Create a new CdawgEdgeWeight with a token but no span info.
    // We use this for searching for an edge that has a certain first token.
    pub fn new(token: u16) -> Self {
        Self {token, start: 0, end: 0}
    }

    pub fn new_full(token: u16, start: usize, end: usize) -> Self {
        Self {token, start, end}
    }

    pub fn get_span(&self) -> (usize, usize) {
        (self.start, self.end)
    }
}

// Note: Compare CdawgEdgeWeight's purely in terms of their associated token.

impl PartialEq for CdawgEdgeWeight {
    fn eq(&self, other: &Self) -> bool {
        self.token == other.token
    }
}

impl PartialOrd for CdawgEdgeWeight {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.token.cmp(&other.token))
    }
}