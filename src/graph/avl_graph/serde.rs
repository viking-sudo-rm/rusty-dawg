use graph::avl_graph::AvlGraph;
use serde::{Serialize, Deserialize};
use serde::ser::{SerializeStruct, Serializer};
use serde::de::Deserializer;
use serde::de::{SeqAccess, Visitor};

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

pub struct AvlGraphVisitor<N, E, Ix, VecN, VecE> {
    pub marker: PhantomData<(N, E, Ix, VecN, VecE)>,
}

impl<'de, N, E, Ix, VecN, VecE> Visitor<'de> for AvlGraphVisitor<N, E, Ix, VecN, VecE>
where
    VecE: Deserialize<'de>,
    VecN: Deserialize<'de>,
    Ix: Deserialize<'de>,
{
    type Value = AvlGraph<N, E, Ix, VecN, VecE>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("struct AvlGraph")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let nodes: VecN = seq.next_element()?.ok_or_else(|| {
            serde::de::Error::invalid_length(0, &self)
        })?;

        let edges: VecE = seq.next_element()?.ok_or_else(|| {
            serde::de::Error::invalid_length(1, &self)
        })?;

        Ok(AvlGraph { nodes, edges, marker: PhantomData })
    }
}