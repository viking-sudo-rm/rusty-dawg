use crate::graph::indexing::{EdgeIndex, NodeIndex};

pub trait EdgeRef<E, Ix> {
    fn get_weight(self) -> E;

    fn get_target(self) -> NodeIndex<Ix>;
}

pub trait NodeRef<N, Ix> {
    fn get_weight(self) -> N
    where
        N: Clone;
    fn get_length(self) -> u64;
    fn get_failure(self) -> Option<NodeIndex<Ix>>;
    fn get_count(self) -> usize;
    fn get_first_edge(self) -> EdgeIndex<Ix>;
}
