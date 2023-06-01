extern crate petgraph;
extern crate type_layout;

// use petgraph::graph::NodeIndex;
use vec_graph::indexing::NodeIndex;

pub trait Weight {
    fn create<W: Weight>(index: u64, length: u64, failure: Option<NodeIndex>) -> W;
    fn extend<W: Weight>(last: &BasicWeight) -> W;
    fn get_length(&self) -> u32;
    fn get_failure(&self) -> Option<NodeIndex>;
    fn set_failure(&mut self, failure: Option<NodeIndex>);
    fn update(&mut self);
}

// #[repr(C)]  // Lay out as written.
// 12 bits instead of 10??
#[derive(Debug)]
pub struct BasicWeight {
    // TODO: Use bitfields here to get 10 bytes.
    length1: u8,
    length2: u32,
    failure1: u8,
    failure2: u32,
    // solid: bool,
    count: u32,
}

impl BasicWeight {
    pub fn new(length: u64, failure: Option<NodeIndex>, count: u64) -> Self {
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

    pub fn initial() -> Self {
        Self::new(0, None, 0)
    }

    pub fn extend(last: &Self) -> Self {
        Self::new(last.get_length() + 1, None, 0)
    }

    pub fn split(state: &Self, next_state: &Self) -> Self {
        Self::new(state.get_length() + 1, next_state.get_failure(), next_state.get_count())
    }

    pub fn get_length(&self) -> u64 {
        return u64::from(self.length1) << 32 | u64::from(self.length2);
    }

    pub fn set_length(&mut self, length: u64) {
        self.length1 = (length >> 32) as u8;
        self.length2 = length as u32;
    }

    pub fn get_failure(&self) -> Option<NodeIndex> {
        if self.failure1 == u8::MAX && self.failure2 == u32::MAX {
            return None;
        }
        let idx = u64::from(self.failure1) << 32 | u64::from(self.failure2);
        return Some(NodeIndex::new(idx as usize));
    }

    pub fn set_failure(&mut self, failure: Option<NodeIndex>) {
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

    // pub fn is_solid(&self) -> bool {
    //     self.solid
    // }

    pub fn increment_count(&mut self) {
        self.count += 1;
    }

    pub fn get_count(&self) -> u64 {
        self.count as u64
    }
}

// #[derive(Debug)]
// pub struct CounterWeight {
//     // TODO: Generic counter size?
//     length: u32,
//     failure: Option<NodeIndex>,
//     counter: u8,
// }

// impl Weight for CounterWeight {
//     pub fn create<CounterWeight>(index: u64, length: u64, failure: Option<NodeIndex>) -> CounterWeight {
//         Self {
//             length: length.into(),
//             failure: failure,
//             counter: 0
//         }
//     }

//     pub fn extend<CounterWeight>(last: NodeIndex) -> CounterWeight {
//         Self {
//             // index: self.dawg[last].index + 1,
//             length: self.dawg[last].get_length() + 1,
//             failure: None,
//             counter: 0,
//         }
//     }

//     pub fn get_length(&self) -> u64 {
//         return self.length.into();
//     }

//     pub fn get_failure(&self) -> Option<NodeIndex> {
//         return self.failure;
//     }

//     pub fn set_failure(&self, failure: Option<NodeIndex>) {
//         self.failure = failure;
//     }

//     pub fn update(&self) {
//         self.counter += 1;
//     }
// }

// #[derive(Debug)]
// struct Weight {
//     index: u32,  // Can drop index field if we don't care about recovering location.
//     length: u32,  // Required for algo, but can throw out after building.
//     failure: Option<NodeIndex>,  // TODO: Required for algo, but make into u32?
//     count: u8,  // Can drop if we don't care about recovering n-gram counts.
// }

#[cfg(test)]
mod tests {

    use BasicWeight;

    #[test]
    fn test_length() {
        let weight = BasicWeight::new(53, None, 0);
        assert_eq!(weight.get_length(), 53);
    }

    #[test]
    fn test_length_overflow() {
        let weight = BasicWeight::new(1 << 35, None, 0);
        assert_eq!(weight.get_length(), 1 << 35);
    }

}