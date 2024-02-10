pub mod cdawg_edge_weight;  // Refered to in higher level types.
mod topological_counter;  // Traverses a built CDAWG to add counts to the states.
// mod crochemore;  // Algo from "On Compact Directed Acyclic Word Graphs"
mod stack;
mod inenaga;  // Algo from "On-line construction of compact directed acyclic word graphs"
mod metadata;
mod token_backing;
pub mod comparator;
pub mod cdawg_state;

// We will use the Inenaga implementation of the build algorithm.
pub use self::inenaga::Cdawg;
pub use self::topological_counter::TopologicalCounter;