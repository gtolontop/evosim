//! Command-line interface definition for the `evosim` binary.
//!
//! Parsed by [`clap`] via `#[derive(Parser)]`. Use `Cli::parse()` in `main`.

use clap::{Parser, Subcommand};

/// Root CLI entry-point for **evosim**.
#[derive(Parser)]
#[command(name = "evosim")]
#[command(about = "Evolutionary creature simulator — grow life from chaos")]
#[command(version)]
pub struct Cli {
    /// The subcommand to execute.
    #[command(subcommand)]
    pub command: Command,
}

/// Available subcommands.
#[derive(Subcommand)]
pub enum Command {
    /// Run headless evolution for the configured number of generations.
    Run {
        /// Path to the TOML config file (created with defaults if absent).
        #[arg(short, long, default_value = "config.toml")]
        config: String,

        /// Override the number of generations from the config.
        #[arg(short, long)]
        generations: Option<usize>,

        /// Override the population size from the config.
        #[arg(short, long)]
        population: Option<usize>,

        /// Override the RNG seed from the config for reproducible runs.
        #[arg(long)]
        seed: Option<u64>,

        /// Directory to write champion JSON files into.
        #[arg(long, default_value = "champions")]
        output: String,
    },

    /// Write a default `config.toml` to disk and exit.
    Init {
        /// Destination path for the generated config file.
        #[arg(short, long, default_value = "config.toml")]
        output: String,
    },

    /// Print a summary of saved champion genomes from a previous run.
    Inspect {
        /// Directory containing `gen_XXXX.json` champion files.
        #[arg(short, long, default_value = "champions")]
        dir: String,

        /// Print only the single best champion instead of the full table.
        #[arg(long)]
        best: bool,
    },
}
