//! # evosim CLI
//!
//! Main entry point for the **evosim** evolution simulator.
//!
//! This binary orchestrates the core physics engine, genetic algorithm, and
//! optional Bevy renderer to evolve soft-body creatures that learn to walk.

use evosim_core::World;
use evosim_genetics::Genome;
use serde::Deserialize;

/// Top-level configuration loaded from a TOML file.
#[derive(Debug, Deserialize)]
pub struct Config {
    /// Fixed physics time-step in seconds.
    pub dt: f32,
    /// Number of creatures per generation.
    pub population_size: u32,
    /// Per-gene mutation probability.
    pub mutation_rate: f32,
    /// Tournament size for selection.
    pub tournament_k: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            dt: 1.0 / 60.0,
            population_size: 100,
            mutation_rate: 0.05,
            tournament_k: 3,
        }
    }
}

fn main() {
    let config = Config::default();

    let world = World::new(config.dt);

    println!("evosim v{}", env!("CARGO_PKG_VERSION"));
    println!(
        "Starting simulation: generation {}, dt = {:.4}s, population = {}",
        world.generation, world.dt, config.population_size
    );

    // Verify genetics crate is linked
    let _genome = Genome::new(vec![0.0; 10]);

    // Verify renderer crate is linked
    let _plugin = evosim_renderer::RendererPlugin;

    println!("All systems initialized. Simulation loop not yet implemented.");
}
