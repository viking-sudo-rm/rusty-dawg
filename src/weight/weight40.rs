use serde::{Deserialize, Serialize};
use std::clone::Clone;

use crate::graph::indexing::{DefaultIx, IndexType, NodeIndex};
use crate::weight::Weight;

pub type DefaultWeight = WeightMinimal;

#[derive(Debug, Deserialize, Serialize, Clone, Copy, Default)]
pub struct WeightMinimal {
    length: DefaultIx,
    failure: DefaultIx,
    count: DefaultIx,
}

impl Weight for WeightMinimal {
    fn new(length: u64, failure: Option<NodeIndex>, count: usize) -> Self {
        Self {
            length: DefaultIx::new(length as usize),
            //length: length as DefaultIx,
            failure: match failure {
                Some(f) => DefaultIx::new(f.index()),
                None => DefaultIx::max_value(),
            },
            count: DefaultIx::new(count),
        }
    }

    fn get_length(&self) -> u64 {
        self.length.index() as u64
    }

    fn set_length(&mut self, length: u64) {
        self.length = DefaultIx::new(length as usize);
    }

    fn get_failure(&self) -> Option<NodeIndex> {
        if self.failure == DefaultIx::max_value() {
            return None;
        }
        Some(NodeIndex::new(self.failure.index()))
    }

    fn set_failure(&mut self, failure: Option<NodeIndex>) {
        match failure {
            Some(f) => self.failure = DefaultIx::new(f.index()),
            None => self.failure = DefaultIx::max_value(),
        }
    }

    fn increment_count(&mut self) {
        self.count = DefaultIx::new(self.count.index() + 1);
    }

    fn get_count(&self) -> usize {
        self.count.index()
    }

    fn set_count(&mut self, count: usize) {
        self.count = DefaultIx::new(count);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_length_weight40() {
        let weight = WeightMinimal::new(53, None, 0);
        assert_eq!(weight.get_length(), 53);
    }

    #[test]
    fn test_length_overflow_weight40() {
        let weight = WeightMinimal::new(1 << 35, None, 0);
        assert_eq!(weight.get_length(), 1 << 35);
    }
}
