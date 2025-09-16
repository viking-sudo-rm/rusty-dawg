mod topological_counter; // Traverses a built CDAWG to add counts to the states.
pub mod traverse_arity;

pub mod array_cdawg;
pub mod cdawg_state;
pub mod comparator;
mod inenaga; // Algo from "On-line construction of compact directed acyclic word graphs"
mod metadata;
pub mod readable_cdawg;
mod stack;
pub mod token_backing;

use crate::cdawg::token_backing::TokenBacking;
use std::cell::RefCell;
use std::rc::Rc;
// We will use the Inenaga implementation of the build algorithm.
pub use self::inenaga::Cdawg;
pub use self::topological_counter::TopologicalCounter;

pub type TokenBackingReference = Rc<RefCell<dyn TokenBacking<u16>>>;
