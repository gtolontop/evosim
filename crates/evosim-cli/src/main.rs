//! # evosim CLI
//!
//! Entry point for the **evosim** evolutionary creature simulator.
//!
//! Dispatches to one of three subcommands:
//! - `run`     — headless evolution loop
//! - `init`    — write a default `config.toml`
//! - `inspect` — display saved champion genomes

mod champions;
mod cli;
mod config;
mod inspect;
mod stats;

use std::time::Instant;

use clap::Parser;
use evosim_core::{Creature, CreatureFactory, Muscle, Particle, World};
use evosim_genetics::{next_generation, Genome};
use rand::SeedableRng;
use rand::rngs::StdRng;

use champions::save_champion;
use cli::{Cli, Command};
use config::load_or_create;
use stats::GenerationStats;

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Run { config, generations, population, seed, output } => {
            cmd_run(config, generations, population, seed, output);
        }
        Command::Init { output } => {
            cmd_init(output);
        }
        Command::Inspect { dir, best } => {
            cmd_inspect(dir, best);
        }
    }
}

// ── Subcommand implementations ───────────────────────────────────────────────

/// Runs the headless evolution loop.
fn cmd_run(
    config_path: String,
    generations: Option<usize>,
    population: Option<usize>,
    seed: Option<u64>,
    output: String,
) {
    let mut cfg = load_or_create(&config_path);
    cfg.apply_overrides(generations, population, seed);

    let genome_len = CreatureFactory::min_genome_len();

    println!("evosim v{}", env!("CARGO_PKG_VERSION"));
    println!(
        "pop={} gens={} steps={} dt={:.4} seed={}",
        cfg.population_size, cfg.generations, cfg.sim_steps, cfg.dt, cfg.seed
    );

    let mut rng = StdRng::seed_from_u64(cfg.seed);

    let mut population: Vec<Genome> = (0..cfg.population_size)
        .map(|_| Genome::random(genome_len, &mut rng))
        .collect();

    let mut baseline_best = f32::NEG_INFINITY;

    for gen in 0..cfg.generations {
        let t_start = Instant::now();

        // (a) Build creatures; failed genomes become dummies with NEG_INFINITY fitness.
        let mut build_errors = 0_usize;
        let mut world = World::new(cfg.dt);
        world.creatures = population
            .iter()
            .map(|genome| match CreatureFactory::build(genome) {
                Ok(creature) => creature,
                Err(_) => {
                    build_errors += 1;
                    make_dummy()
                }
            })
            .collect();

        if build_errors > 0 {
            eprintln!("warning: gen {gen}: {build_errors} genome(s) failed to build");
        }

        // (b) Parallel physics simulation.
        for _ in 0..cfg.sim_steps {
            world.step_all();
        }

        // (c) Collect fitnesses; failed builds get NEG_INFINITY.
        let fitnesses: Vec<f32> = world
            .creatures
            .iter()
            .zip(population.iter())
            .map(|(creature, genome)| {
                if CreatureFactory::build(genome).is_err() {
                    f32::NEG_INFINITY
                } else {
                    creature.fitness
                }
            })
            .collect();

        let elapsed_ms = t_start.elapsed().as_millis() as u64;

        // (d) Stats.
        let stats = GenerationStats::compute(gen as u32, &fitnesses, elapsed_ms);
        stats.print();

        if gen == 0 {
            baseline_best = stats.best_fitness;
        }
        if gen > 0 && gen % 50 == 0 {
            stats.print_milestone(baseline_best);
        }

        // (e) Save champion.
        if let Some(idx) = best_index(&fitnesses) {
            save_champion(&population[idx], gen as u32, fitnesses[idx], &output);
        }

        // (f) Next generation.
        population = next_generation(
            &population,
            &fitnesses,
            cfg.population_size,
            cfg.genetics.mutation_rate,
            cfg.genetics.mutation_strength,
            cfg.genetics.elitism,
            &mut rng,
        );
    }

    println!("Evolution complete.");
}

/// Writes a default config file and exits.
fn cmd_init(output: String) {
    config::write_default(&output);
}

/// Loads and displays champion records from a previous run.
fn cmd_inspect(dir: String, best: bool) {
    let records = inspect::load_champions(&dir);
    if records.is_empty() {
        println!("No champion files found in '{dir}'.");
        return;
    }
    if best {
        inspect::print_best(&records);
    } else {
        inspect::print_summary(&records);
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Returns the index of the highest fitness value, or `None` if empty.
fn best_index(fitnesses: &[f32]) -> Option<usize> {
    fitnesses
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(i, _)| i)
}

/// Creates a minimal two-particle creature with `fitness = NEG_INFINITY` as
/// a placeholder for a genome that failed to build.
fn make_dummy() -> Creature {
    let mut c = Creature::new(
        vec![Particle::new(0.0, 1.0, 1.0), Particle::new(1.0, 1.0, 1.0)],
        vec![Muscle::bone(0, 1, 1.0)],
    );
    c.fitness = f32::NEG_INFINITY;
    c
}
