pub mod cdawg_edge_weight;  // Refered to in higher level types.
// mod crochemore;  // Algo from "On Compact Directed Acyclic Word Graphs"
mod inenaga;  // Algo from "On-line construction of compact directed acyclic word graphs"
mod metadata;
mod token_backing;
pub mod comparator;

// We will use the Inenaga implementation of the build algorithm.
pub use self::inenaga::Cdawg;
