pub use self::weight40::DefaultWeight;
use crate::graph::indexing::NodeIndex;

pub trait Weight {
    fn get_length(&self) -> u64;
    fn set_length(&mut self, length: u64);
    fn get_failure(&self) -> Option<NodeIndex>;
    fn set_failure(&mut self, failure: Option<NodeIndex>);
    fn increment_count(&mut self);
    fn get_count(&self) -> usize;
    fn set_count(&mut self, count: usize);

    fn new(length: u64, failure: Option<NodeIndex>, count: usize) -> Self
    where
        Self: Sized;

    fn initial() -> Self
    where
        Self: Sized,
    {
        Self::new(0, None, 0)
    }

    fn extend(last: &Self) -> Self
    where
        Self: Sized,
    {
        Self::new(last.get_length() + 1, None, 0)
    }

    fn split(state: &Self, next_state: &Self) -> Self
    where
        Self: Sized,
    {
        Self::new(
            state.get_length() + 1,
            next_state.get_failure(),
            next_state.get_count(),
        )
    }
}

pub mod weight40;
