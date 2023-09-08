use graph::indexing::EdgeIndex;

pub trait NodeBacking<N, Ix> {
    fn get_weight(&self) -> &N;

    fn get_weight_mut(&mut self) -> &mut N;

    fn get_first_edge(&self) -> EdgeIndex<Ix>;

    fn set_first_edge(&mut self, first_edge: EdgeIndex<Ix>);
}
