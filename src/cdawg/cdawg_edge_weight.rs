use std::cmp::{Eq, Ord, PartialEq, PartialOrd, Ordering};

#[derive(Eq, Ord, Copy, Clone, Default, Debug)]
pub struct CdawgEdgeWeight {
    pub token: u16,  // TODO: Can remove this and just look it up dynamically as tokens[start]
    pub start: usize,  // TODO: Can make this lower precision.
    pub end: usize,  // TODO: Can make this lower precision.
}

impl CdawgEdgeWeight {
    // Create a new CdawgEdgeWeight with a token but no span info.
    // We use this for searching for an edge that has a certain first token.
    pub fn new_key(token: u16) -> Self {
        Self {token, start: 0, end: 0}
    }

    pub fn new(token: u16, start: usize, end: usize) -> Self {
        Self {token, start, end}
    }

    pub fn get_span(&self) -> (usize, usize) {
        (self.start, self.end)
    }
}

// Can we make custom PartialEq/PartialOrd objects with a pointer to tokens and pass these when we
// search? Would be much more space-efficient. In this case, we could potentially get rid of this
// object and just use a (start, end) tuple (Span).
//
// However, it seems like the LCDAWG extension *requires* the extra token field, so this is probably
// not worth doing.

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