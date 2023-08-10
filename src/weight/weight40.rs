use serde::{Deserialize, Serialize};
use std::clone::Clone;

use crate::weight::Weight;
use graph::indexing::{DefaultIx, IndexType, NodeIndex};

pub type DefaultWeight = WeightMinimal;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WeightMinimal {
    length: DefaultIx,
    failure: DefaultIx,
    count: u32,
}

impl Weight for WeightMinimal {
    fn new(length: u64, failure: Option<NodeIndex>, count: u64) -> Self {
        Self {
            length: DefaultIx::new(length as usize),
            //length: length as DefaultIx,
            failure: match failure {
                Some(f) => DefaultIx::new(f.index()),
                None => DefaultIx::max_value(),
            },
            count: count as u32,
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
        Some(NodeIndex::new(self.failure.index() as usize))
    }

    fn set_failure(&mut self, failure: Option<NodeIndex>) {
        match failure {
            Some(f) => self.failure = DefaultIx::new(f.index()),
            None => self.failure = DefaultIx::max_value(),
        }
    }

    fn increment_count(&mut self) {
        self.count += 1;
    }

    fn get_count(&self) -> u64 {
        return self.count as u64;
    }
}

#[cfg(test)]
mod tests {

    use weight::weight40::WeightMinimal;
    use weight::Weight;

    #[test]
    fn test_length_weight_minimal() {
        let weight = WeightMinimal::new(53, None, 0);
        assert_eq!(weight.get_length(), 53);
    }

    #[test]
    fn test_length_overflow_weight_minimal() {
        let weight = WeightMinimal::new(1 << 35, None, 0);
        assert_eq!(weight.get_length(), 1 << 35);
    }
}
