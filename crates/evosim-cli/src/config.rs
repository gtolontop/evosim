//! Simulation configuration loaded from `config.toml`.

use serde::{Deserialize, Serialize};

use evosim_genetics::GeneticsConfig;

/// Top-level configuration for a headless evolution run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimConfig {
    /// Number of creatures in each generation.
    pub population_size: usize,
    /// Total number of generations to evolve.
    pub generations: usize,
    /// Physics steps simulated per creature per generation.
    pub sim_steps: usize,
    /// Fixed physics time-step in seconds.
    pub dt: f32,
    /// Genetic algorithm hyperparameters.
    pub genetics: GeneticsConfig,
    /// RNG seed for fully reproducible runs.
    pub seed: u64,
}

impl Default for SimConfig {
    fn default() -> Self {
        Self {
            population_size: 200,
            generations: 500,
            sim_steps: 600,
            dt: 0.016,
            genetics: GeneticsConfig {
                population_size: 200,
                genome_len: 30,
                mutation_rate: 0.05,
                mutation_strength: 0.1,
                elitism: 5,
                tournament_k: 5,
            },
            seed: 42,
        }
    }
}

/// Loads a [`SimConfig`] from the given TOML file path.
///
/// If the file does not exist, writes the default config to that path and
/// returns the default. Any IO or parse error falls back to the default and
/// prints a warning to stderr.
pub fn load_or_create(path: &str) -> SimConfig {
    match std::fs::read_to_string(path) {
        Ok(contents) => match toml::from_str(&contents) {
            Ok(cfg) => cfg,
            Err(e) => {
                eprintln!("warning: failed to parse {path}: {e}; using defaults");
                SimConfig::default()
            }
        },
        Err(_) => {
            let default = SimConfig::default();
            let toml_str = toml::to_string_pretty(&default)
                .unwrap_or_else(|_| String::new());
            if let Err(e) = std::fs::write(path, &toml_str) {
                eprintln!("warning: could not write default config to {path}: {e}");
            } else {
                println!("Created default config at {path}");
            }
            default
        }
    }
}
