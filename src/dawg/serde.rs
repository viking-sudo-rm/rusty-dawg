use crate::dawg::Dawg;
use serde::de::Deserializer;
use serde::ser::{SerializeStruct, Serializer};
use serde::{Deserialize, Serialize};

use crate::graph::avl_graph::AvlGraph;
use crate::graph::indexing::{IndexType, NodeIndex};
use crate::memory_backing::MemoryBacking;
use serde::de::{SeqAccess, Visitor};

use std::marker::PhantomData;

impl<E, W, Ix, Mb> Serialize for Dawg<E, W, Ix, Mb>
where
    Mb: MemoryBacking<W, E, Ix> + Default,
    Mb::VecE: Serialize,
    Mb::VecN: Serialize,
    Ix: Serialize + IndexType,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("Dawg", 2)?;
        s.serialize_field("dawg", &self.dawg)?;
        s.serialize_field("initial", &self.initial)?;
        s.serialize_field("max_length", &self.max_length)?;
        s.end()
    }
}

impl<'de, E, W, Ix, Mb> Deserialize<'de> for Dawg<E, W, Ix, Mb>
where
    Mb: MemoryBacking<W, E, Ix> + Default,
    Mb::VecE: Deserialize<'de>,
    Mb::VecN: Deserialize<'de>,
    Ix: Deserialize<'de> + IndexType,
{
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        d.deserialize_struct(
            "Dawg",
            &["dawg", "initial", "max_length"],
            DawgVisitor::<E, W, Ix, Mb> {
                marker: PhantomData,
            },
        )
    }
}

pub struct DawgVisitor<E, W, Ix, Mb> {
    marker: PhantomData<(E, W, Ix, Mb)>,
}

impl<'de, E, W, Ix, Mb> Visitor<'de> for DawgVisitor<E, W, Ix, Mb>
where
    Mb: MemoryBacking<W, E, Ix> + Default,
    Mb::VecE: Deserialize<'de>,
    Mb::VecN: Deserialize<'de>,
    Ix: Deserialize<'de> + IndexType,
{
    type Value = Dawg<E, W, Ix, Mb>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("struct Dawg")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let dawg: AvlGraph<W, E, Ix, Mb> = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;

        let initial: NodeIndex<Ix> = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;

        let max_length: Option<u64> = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;

        Ok(Dawg {
            dawg,
            initial,
            max_length,
        })
    }
}
