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

impl SimConfig {
    /// Applies CLI flag overrides on top of the loaded config values.
    ///
    /// Each argument is applied only when `Some`; `None` leaves the existing
    /// config value unchanged. When `population` is overridden the internal
    /// [`GeneticsConfig::population_size`] is kept in sync automatically.
    pub fn apply_overrides(
        &mut self,
        generations: Option<usize>,
        population: Option<usize>,
        seed: Option<u64>,
    ) {
        if let Some(g) = generations {
            self.generations = g;
        }
        if let Some(p) = population {
            self.population_size = p;
            self.genetics.population_size = p;
        }
        if let Some(s) = seed {
            self.seed = s;
        }
    }
}

impl Default for SimConfig {
    fn default() -> Self {
        Self {
            population_size: 1000,
            generations: 500,
            sim_steps: 400,
            dt: 0.016,
            genetics: GeneticsConfig {
                population_size: 1000,
                genome_len: 40,
                mutation_rate: 0.15,
                mutation_strength: 0.3,
                elitism: 10,
                tournament_k: 5,
                selection_mode: "asexual_top50".to_string(),
            },
            seed: 42,
        }
    }
}

/// Writes the default [`SimConfig`] as TOML to `path`, overwriting any
/// existing file.
///
/// Prints a confirmation or warning to stdout/stderr.
pub fn write_default(path: &str) {
    let default = SimConfig::default();
    match toml::to_string_pretty(&default) {
        Ok(toml_str) => match std::fs::write(path, toml_str) {
            Ok(()) => println!("Wrote default config to '{path}'"),
            Err(e) => eprintln!("error: could not write config to '{path}': {e}"),
        },
        Err(e) => eprintln!("error: could not serialise default config: {e}"),
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
