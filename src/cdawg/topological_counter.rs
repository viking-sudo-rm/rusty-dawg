use anyhow::Result;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::cdawg::inenaga::Cdawg;
use crate::cdawg::stack::Stack;
use crate::graph::indexing::{IndexType, NodeIndex};
use crate::memory_backing::{DiskVec, MemoryBacking};
use crate::weight::Weight;

/// An state on the stack, that should either be opened or closed.
#[derive(Default, Deserialize, Serialize)]
pub struct StackOp<Ix> {
    state: NodeIndex<Ix>,
    open: bool,
}

impl<Ix> StackOp<Ix> {
    pub fn open(state: NodeIndex<Ix>) -> Self {
        Self { state, open: true }
    }

    pub fn close(state: NodeIndex<Ix>) -> Self {
        Self { state, open: false }
    }
}

pub struct TopologicalCounter<Sb> {
    stack: Sb,
}

impl<Ix> TopologicalCounter<Vec<StackOp<Ix>>>
where
    Ix: IndexType + Serialize + for<'de> Deserialize<'de>,
{
    pub fn new_ram() -> Self {
        Self { stack: Vec::new() }
    }
}

impl<Ix> TopologicalCounter<DiskVec<StackOp<Ix>>>
where
    Ix: IndexType + Serialize + for<'de> Deserialize<'de>,
{
    pub fn new_disk<P: AsRef<Path> + std::fmt::Debug>(path: P, capacity: usize) -> Result<Self> {
        let stack = DiskVec::new(path, capacity)?;
        Ok(Self { stack })
    }
}

impl<Sb> TopologicalCounter<Sb> {
    /// DFS implementation of graph traversal.
    pub fn fill_counts<Ix, W, Mb>(&mut self, cdawg: &mut Cdawg<W, Ix, Mb>)
    where
        Ix: IndexType + Serialize + for<'de> Deserialize<'de>,
        W: Weight + Serialize + for<'de> Deserialize<'de> + Clone,
        Mb: MemoryBacking<W, (Ix, Ix), Ix>,
        Sb: Stack<StackOp<Ix>>,
    {
        self.stack.push(StackOp::open(cdawg.get_source()));
        while let Some(op) = self.stack.pop() {
            if op.open {
                // Opening! Set as visited and process all children.
                if cdawg.get_count(op.state) > 0 {
                    continue;
                }
                cdawg.set_count(op.state, 1);
                let neighbors: Vec<_> = cdawg.get_graph().neighbors(op.state).collect();
                self.stack.push(StackOp::close(op.state));
                for next_state in neighbors {
                    self.stack.push(StackOp::open(next_state));
                }
            } else {
                // Closing! Set counts appropriately after all children have been handled.
                let neighbors: Vec<_> = cdawg.get_graph().neighbors(op.state).collect();
                let mut count = 0;
                for next_state in neighbors {
                    count += cdawg.get_count(next_state);
                }
                cdawg.set_count(op.state, count);
            }
        }
    }
}

#[cfg(test)]
#[allow(unused_variables)]
#[allow(unused_imports)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn test_counts_cocoa() {
        let (c, o, a) = (0, 1, 2);
        let mut cdawg: Cdawg = Cdawg::new(Rc::new(RefCell::new(vec![c, o, c, o, a, u16::MAX])));
        cdawg.build();
        let mut counter = TopologicalCounter::new_ram();
        counter.fill_counts(&mut cdawg);

        assert_eq!(cdawg.get_count(NodeIndex::new(0)), 6);
        assert_eq!(cdawg.get_count(NodeIndex::new(1)), 1);
        assert_eq!(cdawg.get_count(NodeIndex::new(2)), 2);
    }

    #[test]
    fn test_counts_abcabcaba() {
        let (a, b, c) = (0, 1, 2);
        let mut cdawg: Cdawg = Cdawg::new(Rc::new(RefCell::new(vec![
            a,
            b,
            c,
            a,
            b,
            c,
            a,
            b,
            a,
            u16::MAX,
        ])));
        cdawg.build();
        let mut counter = TopologicalCounter::new_ram();
        counter.fill_counts(&mut cdawg);

        assert_eq!(cdawg.get_count(NodeIndex::new(0)), 10);
        assert_eq!(cdawg.get_count(NodeIndex::new(1)), 1);
        assert_eq!(cdawg.get_count(NodeIndex::new(2)), 2);
        assert_eq!(cdawg.get_count(NodeIndex::new(3)), 3);
        assert_eq!(cdawg.get_count(NodeIndex::new(4)), 4);
    }

    #[test]
    fn test_counts_multidoc() {
        let (a, b, c) = (0, 1, 2);
        let mut cdawg: Cdawg = Cdawg::new(Rc::new(RefCell::new(vec![
            a,
            b,
            c,
            u16::MAX,
            a,
            u16::MAX,
            b,
            b,
            u16::MAX,
        ])));
        cdawg.build();
        let mut counter = TopologicalCounter::new_ram();
        counter.fill_counts(&mut cdawg);

        assert_eq!(cdawg.node_count(), 7);
        assert_eq!(cdawg.get_count(NodeIndex::new(0)), 9);
        assert_eq!(cdawg.get_count(NodeIndex::new(1)), 1);
        assert_eq!(cdawg.get_count(NodeIndex::new(2)), 1);
        assert_eq!(cdawg.get_count(NodeIndex::new(3)), 2);
        assert_eq!(cdawg.get_count(NodeIndex::new(4)), 1);
        assert_eq!(cdawg.get_count(NodeIndex::new(5)), 3);
    }
}
