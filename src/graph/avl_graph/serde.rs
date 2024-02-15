use crate::graph::avl_graph::AvlGraph;
use crate::graph::indexing::IndexType;
use crate::memory_backing::MemoryBacking;
use serde::de::Deserializer;
use serde::de::{SeqAccess, Visitor};
use serde::ser::{SerializeStruct, Serializer};
use serde::{Deserialize, Serialize};

use std::marker::PhantomData;

impl<N, E, Ix, Mb> Serialize for AvlGraph<N, E, Ix, Mb>
where
    Mb: MemoryBacking<N, E, Ix> + Default,
    Mb::VecE: Serialize,
    Mb::VecN: Serialize,
    Ix: Serialize + IndexType,
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

impl<'de, N, E, Ix, Mb> Deserialize<'de> for AvlGraph<N, E, Ix, Mb>
where
    Mb: MemoryBacking<N, E, Ix> + Default,
    Mb::VecE: Deserialize<'de>,
    Mb::VecN: Deserialize<'de>,
    Ix: Deserialize<'de> + IndexType,
{
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        d.deserialize_struct(
            "AvlGraph",
            &["nodes", "edges"],
            AvlGraphVisitor::<N, E, Ix, Mb> {
                marker: PhantomData,
            },
        )
    }
}

pub struct AvlGraphVisitor<N, E, Ix, Mb> {
    pub marker: PhantomData<(N, E, Ix, Mb)>,
}

impl<'de, N, E, Ix, Mb> Visitor<'de> for AvlGraphVisitor<N, E, Ix, Mb>
where
    Mb: MemoryBacking<N, E, Ix> + Default,
    Mb::VecE: Deserialize<'de>,
    Mb::VecN: Deserialize<'de>,
    Ix: Deserialize<'de> + IndexType,
{
    type Value = AvlGraph<N, E, Ix, Mb>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("struct AvlGraph")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let nodes: Mb::VecN = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;

        let edges: Mb::VecE = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;

        Ok(AvlGraph {
            nodes,
            edges,
            marker: PhantomData,
        })
    }
}
