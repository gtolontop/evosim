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
use evosim_core::{Creature, CreatureFactory, Muscle, Particle};
use evosim_genetics::{next_generation, next_generation_asexual, Genome};
use rand::SeedableRng;
use rand::rngs::StdRng;
use rayon::prelude::*;

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
        Command::Watch { dir, generation } => {
            cmd_watch(dir, generation);
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

    let genome_len = cfg.genetics.genome_len;

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

        // (a) Build creatures in parallel; failed genomes become dummies.
        let build_results: Vec<Option<Creature>> = population
            .par_iter()
            .map(|genome| CreatureFactory::build(genome).ok())
            .collect();

        let build_errors = build_results.iter().filter(|r| r.is_none()).count();
        if build_errors > 0 {
            eprintln!("warning: gen {gen}: {build_errors} genome(s) failed to build");
        }

        // (b) Run ALL sim_steps per creature in a single rayon dispatch.
        //     Each creature runs its full evaluation independently — no sync barriers.
        let dt = cfg.dt;
        let steps = cfg.sim_steps;
        let mut creatures: Vec<Creature> = build_results
            .into_iter()
            .map(|opt| opt.unwrap_or_else(make_dummy))
            .collect();

        creatures.par_iter_mut().for_each(|creature| {
            for _ in 0..steps {
                creature.step(dt);
            }
        });

        // (c) Collect fitnesses; dummies already have NEG_INFINITY.
        let fitnesses: Vec<f32> = creatures.iter().map(|c| c.fitness).collect();

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
        population = if cfg.genetics.selection_mode == "asexual_top50" {
            next_generation_asexual(
                &population,
                &fitnesses,
                cfg.population_size,
                cfg.genetics.mutation_rate,
                cfg.genetics.mutation_strength,
                cfg.genetics.elitism,
                &mut rng,
            )
        } else {
            next_generation(
                &population,
                &fitnesses,
                cfg.population_size,
                cfg.genetics.mutation_rate,
                cfg.genetics.mutation_strength,
                cfg.genetics.elitism,
                &mut rng,
            )
        };
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

/// Opens the Bevy renderer with a saved champion.
///
/// Picks the champion with the highest fitness unless `--generation` is given.
/// Prints a helpful error and exits if no champions are found.
fn cmd_watch(dir: String, generation: Option<u32>) {
    let records = inspect::load_champions(&dir);
    if records.is_empty() {
        eprintln!("No champions found. Run `evosim run` first.");
        return;
    }

    let record = if let Some(gen) = generation {
        match records.iter().find(|r| r.generation == gen) {
            Some(r) => r,
            None => {
                let available: Vec<u32> = records.iter().map(|r| r.generation).collect();
                eprintln!(
                    "No champion saved for generation {gen}. Available generations: {available:?}"
                );
                return;
            }
        }
    } else {
        records
            .iter()
            .max_by(|a, b| a.fitness.partial_cmp(&b.fitness).unwrap_or(std::cmp::Ordering::Equal))
            .expect("records is non-empty")
    };

    let genome = Genome::new(record.genes.clone());
    evosim_renderer::run_renderer(&genome, record.generation, record.fitness);
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
