mod topological_counter; // Traverses a built CDAWG to add counts to the states.
pub mod traverse_arity;

pub mod cdawg_state;
pub mod comparator;
mod inenaga; // Algo from "On-line construction of compact directed acyclic word graphs"
mod metadata;
mod stack;
pub mod token_backing;

// We will use the Inenaga implementation of the build algorithm.
pub use self::inenaga::Cdawg;
pub use self::topological_counter::TopologicalCounter;
