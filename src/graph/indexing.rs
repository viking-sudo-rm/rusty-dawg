// See https://docs.rs/petgraph/0.4.13/src/petgraph/graph_impl/mod.rs.html

use std::fmt;
use std::hash::Hash;

use serde::{Deserialize, Serialize};

// Int-like type for indexing nodes and edges.
// u32 breaks down around 10Gi, but u64 uses more memory than necessary
pub type DefaultIx = Index40;

#[derive(
    Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Debug, Serialize, Deserialize,
)]
pub struct Index48 {
    // Index using 48 bits
    // Higher-order bits must be stored first, or the default comparison will be incorrect
    // Using the same type for members saves on padding space
    // size_of(u16 x 3) = 6, but size_of(u32 + u16) = 8
    pub hi: u16,
    pub mid: u16,
    pub lo: u16,
}

#[derive(
    Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Debug, Serialize, Deserialize,
)]
pub struct Index40 {
    // Index using 40 bits
    pub i4: u8,
    pub i3: u8,
    pub i2: u8,
    pub i1: u8,
    pub i0: u8,
}

/// Trait for the unsigned integer type used for node and edge indices.
///
/// # Safety
///
/// Marked `unsafe` because: the trait must faithfully preserve
/// and convert index values.
pub unsafe trait IndexType: Copy + Default + Hash + Ord + fmt::Debug + 'static {
    fn new(x: usize) -> Self;
    fn index(&self) -> usize;
    fn max_value() -> Self;
}

unsafe impl IndexType for usize {
    #[inline(always)]
    fn new(x: usize) -> Self {
        x
    }
    #[inline(always)]
    fn index(&self) -> Self {
        *self
    }
    #[inline(always)]
    fn max_value() -> Self {
        usize::MAX
    }
}

unsafe impl IndexType for Index48 {
    #[inline(always)]
    fn new(x: usize) -> Self {
        Index48 {
            lo: (x & 0xFFFF) as u16,
            mid: ((x >> 16) & 0xFFFF) as u16,
            hi: ((x >> 32) & 0xFFFF) as u16,
        }
    }
    #[inline(always)]
    fn index(&self) -> usize {
        ((self.hi as usize) << 32) | ((self.mid as usize) << 16) | (self.lo as usize)
    }
    #[inline(always)]
    fn max_value() -> Self {
        Index48 {
            lo: u16::MAX,
            mid: u16::MAX,
            hi: u16::MAX,
        }
    }
}

unsafe impl IndexType for Index40 {
    #[inline(always)]
    fn new(x: usize) -> Self {
        Index40 {
            i0: (x & 0xFF) as u8,
            i1: ((x >> 8) & 0xFF) as u8,
            i2: ((x >> 16) & 0xFF) as u8,
            i3: ((x >> 24) & 0xFF) as u8,
            i4: ((x >> 32) & 0xFF) as u8,
        }
    }
    #[inline(always)]
    fn index(&self) -> usize {
        ((self.i4 as usize) << 32)
            | ((self.i3 as usize) << 24)
            | ((self.i2 as usize) << 16)
            | ((self.i1 as usize) << 8)
            | (self.i0 as usize)
    }
    #[inline(always)]
    fn max_value() -> Self {
        Index40 {
            i0: u8::MAX,
            i1: u8::MAX,
            i2: u8::MAX,
            i3: u8::MAX,
            i4: u8::MAX,
        }
    }
}

unsafe impl IndexType for u32 {
    #[inline(always)]
    fn new(x: usize) -> Self {
        x as u32
    }
    #[inline(always)]
    fn index(&self) -> usize {
        *self as usize
    }
    #[inline(always)]
    fn max_value() -> Self {
        u32::MAX
    }
}

unsafe impl IndexType for u16 {
    #[inline(always)]
    fn new(x: usize) -> Self {
        x as u16
    }
    #[inline(always)]
    fn index(&self) -> usize {
        *self as usize
    }
    #[inline(always)]
    fn max_value() -> Self {
        u16::MAX
    }
}

unsafe impl IndexType for u8 {
    #[inline(always)]
    fn new(x: usize) -> Self {
        x as u8
    }
    #[inline(always)]
    fn index(&self) -> usize {
        *self as usize
    }
    #[inline(always)]
    fn max_value() -> Self {
        u8::MAX
    }
}

/// Node identifier.
#[derive(Copy, Clone, Default, PartialEq, PartialOrd, Eq, Ord, Hash, Serialize, Deserialize)]
pub struct NodeIndex<Ix = DefaultIx>(Ix);

impl<Ix: IndexType> NodeIndex<Ix> {
    #[inline]
    pub fn new(x: usize) -> Self {
        NodeIndex(IndexType::new(x))
    }

    #[inline]
    pub fn index(self) -> usize {
        self.0.index()
    }

    #[inline]
    pub fn end() -> Self {
        NodeIndex(IndexType::max_value())
    }

    fn _into_edge(self) -> EdgeIndex<Ix> {
        EdgeIndex(self.0)
    }
}

impl<Ix: IndexType> From<Ix> for NodeIndex<Ix> {
    fn from(ix: Ix) -> Self {
        NodeIndex(ix)
    }
}

impl<Ix: fmt::Debug> fmt::Debug for NodeIndex<Ix> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "NodeIndex({:?})", self.0)
    }
}

/// Short version of `NodeIndex::new`
pub fn node_index<Ix: IndexType>(index: usize) -> NodeIndex<Ix> {
    NodeIndex::new(index)
}

/// Short version of `EdgeIndex::new`
pub fn edge_index<Ix: IndexType>(index: usize) -> EdgeIndex<Ix> {
    EdgeIndex::new(index)
}

/// Edge identifier.
#[derive(Copy, Clone, Default, PartialEq, PartialOrd, Eq, Ord, Hash, Serialize, Deserialize)]
pub struct EdgeIndex<Ix = DefaultIx>(Ix);

impl<Ix: IndexType> EdgeIndex<Ix> {
    #[inline]
    pub fn new(x: usize) -> Self {
        EdgeIndex(IndexType::new(x))
    }

    #[inline]
    pub fn index(self) -> usize {
        self.0.index()
    }

    /// An invalid `EdgeIndex` used to denote absence of an edge, for example
    /// to end an adjacency list.
    #[inline]
    pub fn end() -> Self {
        EdgeIndex(Ix::max_value())
    }

    fn _into_node(self) -> NodeIndex<Ix> {
        NodeIndex(self.0)
    }
}

impl<Ix: fmt::Debug> fmt::Debug for EdgeIndex<Ix> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "EdgeIndex({:?})", self.0)
    }
}
