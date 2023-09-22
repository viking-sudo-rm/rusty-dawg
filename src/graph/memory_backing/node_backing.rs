use graph::indexing::EdgeIndex;
use weight::weight_mutator::WeightMutator;

pub trait NodeBacking<N, Ix> {
    type WeightMut<'a>: WeightMutator<N>
    where
        Self: 'a;

    fn new(weight: N) -> Self;

    fn get_weight(&self) -> &N;

    fn get_weight_mut(&mut self) -> Self::WeightMut<'_>;

    fn get_first_edge(&self) -> EdgeIndex<Ix>;

    fn set_first_edge(&mut self, first_edge: EdgeIndex<Ix>);
}
