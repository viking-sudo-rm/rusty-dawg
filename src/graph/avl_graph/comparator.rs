use comparator::Comparator;
use std::cmp::Ordering;

pub const DEFAULT_CMP: DefaultComparator = DefaultComparator {};

pub struct DefaultComparator {}

impl<E: Eq + Ord> Comparator<E> for DefaultComparator {
    fn compare(&self, e1: &E, e2: &E) -> Ordering {
        e1.cmp(e2)
    }
}
