use graph::avl_graph::AvlGraph;
use serde::{Serialize, Deserialize};
use serde::ser::{SerializeStruct, Serializer};
use serde::de::Deserializer;

use graph::avl_graph::avl_graph_visitor::AvlGraphVisitor;
use std::marker::PhantomData;

impl<N, E, Ix, VecN, VecE> Serialize for AvlGraph<N, E, Ix, VecN, VecE>
where
    VecE: Serialize,
    VecN: Serialize,
    Ix: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("AvlGraph", 2)?;
        s.serialize_field("nodes", &self.nodes)?;
        s.serialize_field("edges", &self.edges)?;
        s.end()
    }
}

impl<'de, N, E, Ix, VecN, VecE> Deserialize<'de> for AvlGraph<N, E, Ix, VecN, VecE>
where
    VecE: Deserialize<'de>,
    VecN: Deserialize<'de>,
    Ix: Deserialize<'de>,
{
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        d.deserialize_struct("AvlGraph", &["nodes", "edges"], AvlGraphVisitor::<N, E, Ix, VecN, VecE> {
            marker: PhantomData,
        })
    }
}