// See https://docs.rs/petgraph/0.4.13/src/petgraph/graph_impl/mod.rs.html

use std::fmt;
use std::hash::Hash;

use serde::{Serialize, Deserialize};

// u40 would be nice here up to 1T tokens. u32 breaks down around 10Gi 
pub type DefaultIx = usize;

/// Trait for the unsigned integer type used for node and edge indices.
///
/// Marked `unsafe` because: the trait must faithfully preseve
/// and convert index values.
pub unsafe trait IndexType : Copy + Default + Hash + Ord + fmt::Debug + 'static
{
    fn new(x: usize) -> Self;
    fn index(&self) -> usize;
    fn max() -> Self;
}

unsafe impl IndexType for usize {
    #[inline(always)]
    fn new(x: usize) -> Self { x }
    #[inline(always)]
    fn index(&self) -> Self { *self }
    #[inline(always)]
    fn max() -> Self { ::std::usize::MAX }
}

unsafe impl IndexType for u32 {
    #[inline(always)]
    fn new(x: usize) -> Self { x as u32 }
    #[inline(always)]
    fn index(&self) -> usize { *self as usize }
    #[inline(always)]
    fn max() -> Self { ::std::u32::MAX }
}

unsafe impl IndexType for u16 {
    #[inline(always)]
    fn new(x: usize) -> Self { x as u16 }
    #[inline(always)]
    fn index(&self) -> usize { *self as usize }
    #[inline(always)]
    fn max() -> Self { ::std::u16::MAX }
}

unsafe impl IndexType for u8 {
    #[inline(always)]
    fn new(x: usize) -> Self { x as u8 }
    #[inline(always)]
    fn index(&self) -> usize { *self as usize }
    #[inline(always)]
    fn max() -> Self { ::std::u8::MAX }
}

/// Node identifier.
#[derive(Copy, Clone, Default, PartialEq, PartialOrd, Eq, Ord, Hash, Serialize, Deserialize)]
pub struct NodeIndex<Ix=DefaultIx>(Ix);

impl<Ix: IndexType> NodeIndex<Ix>
{
    #[inline]
    pub fn new(x: usize) -> Self {
        NodeIndex(IndexType::new(x))
    }

    #[inline]
    pub fn index(self) -> usize
    {
        self.0.index()
    }

    #[inline]
    pub fn end() -> Self
    {
        NodeIndex(IndexType::max())
    }

    fn _into_edge(self) -> EdgeIndex<Ix> {
        EdgeIndex(self.0)
    }
}

impl<Ix: IndexType> From<Ix> for NodeIndex<Ix> {
    fn from(ix: Ix) -> Self { NodeIndex(ix) }
}

impl<Ix: fmt::Debug> fmt::Debug for NodeIndex<Ix>
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "NodeIndex({:?})", self.0)
    }
}

/// Short version of `NodeIndex::new`
pub fn node_index<Ix: IndexType>(index: usize) -> NodeIndex<Ix> { NodeIndex::new(index) }

/// Short version of `EdgeIndex::new`
pub fn edge_index<Ix: IndexType>(index: usize) -> EdgeIndex<Ix> { EdgeIndex::new(index) }

/// Edge identifier.
#[derive(Copy, Clone, Default, PartialEq, PartialOrd, Eq, Ord, Hash, Serialize, Deserialize)]
pub struct EdgeIndex<Ix=DefaultIx>(Ix);

impl<Ix: IndexType> EdgeIndex<Ix>
{
    #[inline]
    pub fn new(x: usize) -> Self {
        EdgeIndex(IndexType::new(x))
    }

    #[inline]
    pub fn index(self) -> usize
    {
        self.0.index()
    }

    /// An invalid `EdgeIndex` used to denote absence of an edge, for example
    /// to end an adjacency list.
    #[inline]
    pub fn end() -> Self {
        EdgeIndex(IndexType::max())
    }

    fn _into_node(self) -> NodeIndex<Ix> {
        NodeIndex(self.0)
    }
}

impl<Ix: fmt::Debug> fmt::Debug for EdgeIndex<Ix>
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "EdgeIndex({:?})", self.0)
    }
}