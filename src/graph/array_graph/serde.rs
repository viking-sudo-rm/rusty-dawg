use crate::graph::indexing::IndexType;
use crate::memory_backing::ArrayMemoryBacking;
use serde::de::Deserializer;
use serde::de::{SeqAccess, Visitor};
use serde::ser::{SerializeStruct, Serializer};
use serde::{Deserialize, Serialize};

use crate::graph::array_graph::ArrayGraph;
use std::marker::PhantomData;

impl<N, E, Ix, Mb> Serialize for ArrayGraph<N, E, Ix, Mb>
where
    Mb: ArrayMemoryBacking<N, E, Ix> + Default,
    Mb::ArrayVecE: Serialize,
    Mb::ArrayVecN: Serialize,
    Ix: Serialize + IndexType,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("ArrayGraph", 2)?;
        s.serialize_field("nodes", &self.nodes)?;
        s.serialize_field("edges", &self.edges)?;
        s.end()
    }
}

impl<'de, N, E, Ix, Mb> Deserialize<'de> for ArrayGraph<N, E, Ix, Mb>
where
    Mb: ArrayMemoryBacking<N, E, Ix> + Default,
    Mb::ArrayVecE: Deserialize<'de>,
    Mb::ArrayVecN: Deserialize<'de>,
    Ix: Deserialize<'de> + IndexType,
{
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        d.deserialize_struct(
            "ArrayGraph",
            &["nodes", "edges"],
            ArrayGraphVisitor::<N, E, Ix, Mb> {
                marker: PhantomData,
            },
        )
    }
}

pub struct ArrayGraphVisitor<N, E, Ix, Mb> {
    pub marker: PhantomData<(N, E, Ix, Mb)>,
}

impl<'de, N, E, Ix, Mb> Visitor<'de> for ArrayGraphVisitor<N, E, Ix, Mb>
where
    Mb: ArrayMemoryBacking<N, E, Ix> + Default,
    Mb::ArrayVecE: Deserialize<'de>,
    Mb::ArrayVecN: Deserialize<'de>,
    Ix: Deserialize<'de> + IndexType,
{
    type Value = ArrayGraph<N, E, Ix, Mb>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("struct ArrayGraph")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let nodes: Mb::ArrayVecN = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;

        let edges: Mb::ArrayVecE = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;

        Ok(ArrayGraph { nodes, edges })
    }
}
