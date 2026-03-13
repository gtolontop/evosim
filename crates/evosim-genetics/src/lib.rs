//! # evosim-genetics
//!
//! Genetic algorithm primitives for the **evosim** evolution simulator.
//!
//! This crate provides genome encoding, mutation operators, and selection
//! strategies. It is deliberately decoupled from the physics engine so that
//! genetic operations can be tested and evolved independently.

pub mod genome;
pub mod mutation;
pub mod selection;

pub use genome::Genome;
pub use mutation::mutate;
pub use selection::tournament_select;
