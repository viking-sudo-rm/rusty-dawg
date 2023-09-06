use serde::Deserialize;
use serde::de::{SeqAccess, Visitor};

use graph::avl_graph::AvlGraph;
use graph::avl_graph::node::Node;
use graph::avl_graph::edge::Edge;

pub struct AvlGraphVisitor<N, E, Ix> {
    pub marker: std::marker::PhantomData<AvlGraph<N, E, Ix>>,
}

impl<'de, N, E, Ix> Visitor<'de> for AvlGraphVisitor<N, E, Ix>
where
    E: Deserialize<'de>,
    N: Deserialize<'de>,
    Ix: Deserialize<'de>,
{
    type Value = AvlGraph<N, E, Ix>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("struct AvlGraph")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let nodes: Vec<Node<N, Ix>> = seq.next_element()?.ok_or_else(|| {
            serde::de::Error::invalid_length(0, &self)
        })?;

        let edges: Vec<Edge<E, Ix>> = seq.next_element()?.ok_or_else(|| {
            serde::de::Error::invalid_length(1, &self)
        })?;

        Ok(AvlGraph { nodes, edges })
    }
}