use std::fmt::{Debug, Formatter, Result};
use std::cmp::{Eq, Ord};

use vec_graph::Graph;
use vec_graph::IndexType;
use vec_graph::indexing::NodeIndex;

pub struct Dot<'a, N, E, Ix> {
    graph: &'a Graph<N, E, Ix>,
}

impl<'a, N, E, Ix> Dot<'a, N, E, Ix> {
    pub fn new(graph: &'a Graph<N, E, Ix>) -> Self {
        Self {graph: graph}
    }
}

impl<'a, N: Debug, E: Eq + Ord + Copy + Debug, Ix: IndexType> Debug for Dot<'a, N, E, Ix> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "digraph {{\n")?;
        for idx in 0..self.graph.node_count() {
            let node_index = NodeIndex::new(idx);
            let weight = self.graph.node_weight(node_index).unwrap();
            write!(f, "  {} [{:?}]\n", idx, weight)?;
        }
        for idx in 0..self.graph.node_count() {
            let node_index = NodeIndex::new(idx);
            for edge in self.graph.edges(node_index) {
                write!(f, "  {} -> {} [{:?}]\n", idx, edge.target().index(), edge.weight())?;
            }
        }
        write!(f, "}}")
    }
}

#[cfg(test)]
mod tests {
    use vec_graph::Graph;
    use dot::Dot;

    #[test]
    fn test_print_graph() {
        let mut graph: Graph<u8, u16> = Graph::new();
        let q0 = graph.add_node(5);
        let q1 = graph.add_node(6);
        graph.add_edge(q0, q1, 7);

        let dot = Dot::new(&graph);
        println!("{:?}", dot);
        assert_eq!(format!("{dot:?}"), "digraph {\n  0 [5]\n  1 [6]\n  0 -> 1 [7]\n}");
    }

}