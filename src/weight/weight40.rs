use serde::{Deserialize, Serialize};
use std::clone::Clone;

use crate::weight::Weight;
use graph::indexing::NodeIndex;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Weight40 {
    // TODO: Use bitfields here to get 10 bytes.
    length1: u8,
    length2: u32,
    failure1: u8,
    failure2: u32,
    count: u32,
}

impl Weight for Weight40 {
    fn new(length: u64, failure: Option<NodeIndex>, count: u64) -> Self {
        Self {
            length1: (length >> 32) as u8,
            length2: length as u32,
            failure1: match failure {
                Some(f) => (f.index() >> 32) as u8,
                None => u8::MAX,
            },
            failure2: match failure {
                Some(f) => f.index() as u32,
                None => u32::MAX,
            },
            // solid: solid,
            count: count as u32,
        }
    }

    fn get_length(&self) -> u64 {
        u64::from(self.length1) << 32 | u64::from(self.length2)
    }

    fn set_length(&mut self, length: u64) {
        self.length1 = (length >> 32) as u8;
        self.length2 = length as u32;
    }

    fn get_failure(&self) -> Option<NodeIndex> {
        if self.failure1 == u8::MAX && self.failure2 == u32::MAX {
            return None;
        }
        let idx = u64::from(self.failure1) << 32 | u64::from(self.failure2);
        Some(NodeIndex::new(idx as usize))
    }

    fn set_failure(&mut self, failure: Option<NodeIndex>) {
        match failure {
            Some(f) => {
                self.failure1 = (f.index() >> 32) as u8;
                self.failure2 = f.index() as u32;
            }
            None => {
                self.failure1 = u8::MAX;
                self.failure2 = u32::MAX;
            }
        }
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

    use weight::weight40::Weight40;
    use weight::Weight;

    #[test]
    fn test_length_weight40() {
        let weight = Weight40::new(53, None, 0);
        assert_eq!(weight.get_length(), 53);
    }

    #[test]
    fn test_length_overflow_weight40() {
        let weight = Weight40::new(1 << 35, None, 0);
        assert_eq!(weight.get_length(), 1 << 35);
    }
}
