use serde::Deserialize;
use serde::de::{SeqAccess, Visitor};

use std::marker::PhantomData;

use graph::avl_graph::AvlGraph;

pub struct AvlGraphVisitor<N, E, Ix, VecN, VecE> {
    pub marker: PhantomData<AvlGraph<N, E, Ix, VecN, VecE>>,
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