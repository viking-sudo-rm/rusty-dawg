use graph::indexing::{NodeIndex, EdgeIndex};

pub trait EdgeBacking<E, Ix> {
    fn get_weight(&self) -> &E;

    fn get_target(&self) -> NodeIndex<Ix>;

    fn set_target(&mut self, target: NodeIndex<Ix>);

    fn get_left(&self) -> EdgeIndex<Ix>;

    fn set_left(&mut self, left: EdgeIndex<Ix>);

    fn get_right(&self) -> EdgeIndex<Ix>;

    fn set_right(&mut self, right: EdgeIndex<Ix>);

    fn get_balance_factor(&self) -> i8;

    fn set_balance_factor(&mut self, bf: i8);
}