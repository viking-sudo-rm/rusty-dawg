use comparator::Comparator;
use std::cmp::Ordering;

pub const DEFAULT_CMP: DefaultComparator = DefaultComparator {};

pub struct DefaultComparator {}

impl<E: Eq + Ord> Comparator<E> for DefaultComparator {
    fn compare(&self, e1: &E, e2: &E) -> Ordering {
        if e1 == e2 {
            Ordering::Equal
        } else if e1 < e2 {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    }
}