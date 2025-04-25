use anyhow::Result;
use kdam::{tqdm, BarExt};
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::cdawg::inenaga::Cdawg;
use crate::cdawg::stack::Stack;
use crate::graph::indexing::{IndexType, NodeIndex};
use crate::memory_backing::{DiskVec, MemoryBacking};
use crate::weight::Weight;

/// Based on Topological Counter.
/// TODO: Could standardize names and potentially generalize.
pub struct TraverseArity<Sb> {
    stack: Sb,
    visited: Vec<bool>, // Only support RAM.
}

impl<Ix> TraverseArity<Vec<Ix>>
where
    Ix: IndexType + Serialize + for<'de> Deserialize<'de>,
{
    pub fn new_ram(capacity: usize) -> Self {
        Self {
            stack: Vec::new(),
            visited: vec![false; capacity],
        }
    }
}

impl<Ix> TraverseArity<DiskVec<Ix>>
where
    Ix: IndexType + Serialize + for<'de> Deserialize<'de>,
{
    pub fn new_disk<P: AsRef<Path> + std::fmt::Debug>(path: P, capacity: usize) -> Result<Self> {
        let stack = DiskVec::new(path, capacity)?;
        Ok(Self {
            stack,
            visited: vec![false; capacity],
        })
    }
}

impl<Sb> TraverseArity<Sb> {
    /// DFS implementation of graph traversal.
    pub fn traverse_arity<Ix, W, Mb>(&mut self, cdawg: &mut Cdawg<W, Ix, Mb>) -> Vec<usize>
    where
        Ix: IndexType + Serialize + for<'de> Deserialize<'de>,
        W: Weight + Serialize + for<'de> Deserialize<'de> + Clone,
        Mb: MemoryBacking<W, (Ix, Ix), Ix>,
        Sb: Stack<usize>,
    {
        let mut arities = Vec::with_capacity(self.visited.len());
        self.stack.push(cdawg.get_source().index());

        let mut pb = tqdm!(total = self.visited.len());
        while let Some(state) = self.stack.pop() {
            if self.visited[state.index()] {
                continue;
            }

            let idx = NodeIndex::new(state);
            let mut arity = 0;
            for next_state in cdawg.get_graph().neighbors(idx) {
                arity += 1;
                if !self.visited[next_state.index()] {
                    self.stack.push(next_state.index());
                }
            }
            arities.push(arity);
            self.visited[state] = true;
            let _ = pb.update(1);
        }
        eprintln!();
        arities
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
    fn test_traverse_arities_cocoa() {
        let (c, o, a) = (0, 1, 2);
        let mut cdawg: Cdawg = Cdawg::new(Rc::new(RefCell::new(vec![c, o, c, o, a, u16::MAX])));
        cdawg.build();
        let mut ta = TraverseArity::new_ram(20);
        let arities = ta.traverse_arity(&mut cdawg);
        assert_eq!(arities, vec![4, 2, 1]); // 4 at source, 1 at sink (self loop), 2 at internal
    }
}
