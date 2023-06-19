// extern crate type_layout;
use std::clone::Clone;

use serde::{Serialize, Deserialize};

// use petgraph::graph::NodeIndex;
use graph::indexing::NodeIndex;

pub trait Weight {
    fn get_length(&self) -> u64;
    fn set_length(&mut self, u64);
    fn get_failure(&self) -> Option<NodeIndex>;
    fn set_failure(&mut self, failure: Option<NodeIndex>);
    fn increment_count(&mut self);
    fn get_count(&self) -> u64;

    fn new(length: u64, failure: Option<NodeIndex>, count: u64) -> Self
    where Self: Sized;

    fn initial() -> Self
    where Self: Sized {
        Self::new(0, None, 0)
    }

    fn extend(last: &Self) -> Self
    where Self: Sized {
        Self::new(last.get_length() + 1, None, 0)
    }

    fn split(state: &Self, next_state: &Self) -> Self
    where Self: Sized {
        Self::new(state.get_length() + 1, next_state.get_failure(), next_state.get_count())
    }

}

#[derive(Debug, Deserialize, Serialize, Clone)]
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
        return u64::from(self.length1) << 32 | u64::from(self.length2);
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
        return Some(NodeIndex::new(idx as usize));
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

    use weight::{Weight, BasicWeight, Weight40};

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

    #[test]
    fn test_length_weight32() {
        let weight = BasicWeight::new(53, None, 0);
        assert_eq!(weight.get_length(), 53);
    }

}