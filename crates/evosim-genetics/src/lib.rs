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
pub use mutation::{crossover, mutate};
pub use selection::{next_generation, tournament_select};

/// Configuration for genetic algorithm operations.
///
/// This struct holds all parameters needed for evolution, including population
/// size, genome length, and mutation/selection parameters.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GeneticsConfig {
    /// Number of organisms in the population
    pub population_size: usize,
    /// Number of genes per genome
    pub genome_len: usize,
    /// Per-gene mutation probability, in [0.0, 1.0]
    pub mutation_rate: f32,
    /// Standard deviation of Gaussian noise for mutations
    pub mutation_strength: f32,
    /// Number of best genomes to preserve each generation without mutation
    pub elitism: usize,
    /// Tournament size (k) for selection
    pub tournament_k: usize,
}

impl Default for GeneticsConfig {
    fn default() -> Self {
        Self {
            population_size: 64,
            genome_len: 20,
            mutation_rate: 0.05,
            mutation_strength: 0.1,
            elitism: 5,
            tournament_k: 5,
        }
    }
}
