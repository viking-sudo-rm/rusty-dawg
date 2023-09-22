use graph::indexing::NodeIndex;

use weight::Weight;

pub trait WeightMutator<N> {
    fn set_length(self, length: u64);
    fn set_failure(self, failure: Option<NodeIndex>);
    fn increment_count(self);
}

impl<N: Weight> WeightMutator<N> for &mut N {
    fn set_length(self, length: u64) {
        N::set_length(self, length);
    }

    fn set_failure(self, failure: Option<NodeIndex>) {
        N::set_failure(self, failure);
    }

    fn increment_count(self) {
        N::increment_count(self);
    }
}