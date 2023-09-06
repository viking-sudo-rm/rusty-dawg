use dawg::Dawg;
use serde::{Serialize, Deserialize};
use serde::ser::{SerializeStruct, Serializer};
use serde::de::Deserializer;

use graph::indexing::NodeIndex;
use graph::avl_graph::AvlGraph;
use serde::de::{SeqAccess, Visitor};

use std::marker::PhantomData;

impl<E, W, Ix, VecE, VecW> Serialize for Dawg<E, W, Ix, VecE, VecW>
where
    VecE: Serialize,
    VecW: Serialize,
    Ix: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("Dawg", 2)?;
        s.serialize_field("dawg", &self.dawg)?;
        s.serialize_field("initial", &self.initial)?;
        s.end()
    }
}

impl<'de, E, W, Ix, VecE, VecW> Deserialize<'de> for Dawg<E, W, Ix, VecE, VecW>
where
    VecE: Deserialize<'de>,
    VecW: Deserialize<'de>,
    Ix: Deserialize<'de>,
{
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        d.deserialize_struct("Dawg", &["dawg", "initial"], DawgVisitor::<E, W, Ix, VecE, VecW> {
            marker: PhantomData,
        })
    }
}

pub struct DawgVisitor<E, W, Ix, VecE, VecW> {
    marker: PhantomData<(E, W, Ix, VecE, VecW)>,
}

impl<'de, E, W, Ix, VecE, VecW> Visitor<'de> for DawgVisitor<E, W, Ix, VecE, VecW>
where
    VecE: Deserialize<'de>,
    VecW: Deserialize<'de>,
    Ix: Deserialize<'de>,
{
    type Value = Dawg<E, W, Ix, VecE, VecW>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("struct Dawg")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let dawg: AvlGraph<W, E, Ix, VecW, VecE> = seq.next_element()?.ok_or_else(|| {
            serde::de::Error::invalid_length(0, &self)
        })?;

        let initial: NodeIndex<Ix> = seq.next_element()?.ok_or_else(|| {
            serde::de::Error::invalid_length(1, &self)
        })?;

        Ok(Dawg { dawg, initial })
    }
}