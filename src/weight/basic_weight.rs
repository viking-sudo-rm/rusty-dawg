use std::clone::Clone;

use serde::{Deserialize, Serialize};

// use petgraph::graph::NodeIndex;
use graph::indexing::NodeIndex;

use crate::weight::Weight;

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct BasicWeight {
    length: u32,
    failure: u32,
    count: u32,
}

impl Weight for BasicWeight {
    fn new(length: u64, failure: Option<NodeIndex>, count: u64) -> Self {
        Self {
            length: length as u32,
            failure: match failure {
                Some(f) => f.index() as u32,
                None => u32::MAX,
            },
            count: count as u32,
        }
    }

    fn get_length(&self) -> u64 {
        u64::from(self.length)
    }

    fn set_length(&mut self, length: u64) {
        self.length = length as u32;
    }

    fn get_failure(&self) -> Option<NodeIndex> {
        if self.failure == u32::MAX {
            return None;
        }
        Some(NodeIndex::new(self.failure as usize))
    }

    fn set_failure(&mut self, failure: Option<NodeIndex>) {
        self.failure = match failure {
            Some(f) => f.index() as u32,
            None => u32::MAX,
        };
    }

    fn increment_count(&mut self) {
        self.count += 1;
    }

    fn get_count(&self) -> u64 {
        self.count as u64
    }
}

#[cfg(test)]
mod tests {

    use weight::basic_weight::BasicWeight;
    use weight::Weight;

    #[test]
    fn test_length_weight32() {
        let weight = BasicWeight::new(53, None, 0);
        assert_eq!(weight.get_length(), 53);
    }
}
