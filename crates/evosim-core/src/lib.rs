//! # evosim-core
//!
//! Pure physics engine for the **evosim** evolution simulator.
//!
//! This crate contains the core data types and (future) simulation logic for
//! soft-body creatures built from particles and muscles. It has no graphics
//! dependencies and can run headlessly.

pub mod constants;
pub mod creature;
pub mod factory;
pub mod muscle;
pub mod particle;
pub mod world;

pub use constants::*;
pub use creature::Creature;
pub use factory::CreatureFactory;
pub use muscle::Muscle;
pub use particle::Particle;
pub use world::World;
