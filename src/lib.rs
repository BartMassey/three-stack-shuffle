//! The three-stack split-merge card-sorting machine: heuristics and exact lower
//! bounds, exact search, constructive merge sorters, an inadmissible local-search
//! planner, and the whole-cycle permutation-distance model.
//!
//! A Rust port of the (now reference) Python implementation under `python/`,
//! validated against the same invariants. See `docs/NOTES.md`.

pub mod cycle;
pub mod heuristics;
pub mod machine;
pub mod oct;
pub mod planner;
pub mod search;
pub mod sorters;

pub mod util;
