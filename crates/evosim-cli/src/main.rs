//! # evosim CLI
//!
//! Headless evolution loop for the **evosim** soft-body creature simulator.
//!
//! Loads (or creates) `config.toml`, initialises a random population, and
//! runs the genetic algorithm for the configured number of generations.
//! Champion genomes are saved to `champions/gen_NNNN.json` each generation.

mod champions;
mod config;
mod stats;

use std::time::Instant;

use evosim_core::{Creature, CreatureFactory, Muscle, Particle, World};
use evosim_genetics::{next_generation, Genome};
use rand::SeedableRng;
use rand::rngs::StdRng;

use champions::save_champion;
use config::load_or_create;
use stats::GenerationStats;

fn main() {
    // ── Config ───────────────────────────────────────────────────────────────
    let cfg = load_or_create("config.toml");
    let genome_len = CreatureFactory::min_genome_len();

    println!("evosim v{}", env!("CARGO_PKG_VERSION"));
    println!(
        "pop={} gens={} steps={} dt={:.4} seed={}",
        cfg.population_size, cfg.generations, cfg.sim_steps, cfg.dt, cfg.seed
    );

    // ── Seed RNG ─────────────────────────────────────────────────────────────
    let mut rng = StdRng::seed_from_u64(cfg.seed);

    // ── Initial population ───────────────────────────────────────────────────
    let mut population: Vec<Genome> = (0..cfg.population_size)
        .map(|_| Genome::random(genome_len, &mut rng))
        .collect();

    let mut baseline_best = f32::NEG_INFINITY;

    // ── Evolution loop ───────────────────────────────────────────────────────
    for gen in 0..cfg.generations {
        let t_start = Instant::now();

        // (a) Build creatures; invalid genomes produce a dummy with NEG_INFINITY
        //     fitness so they are eliminated naturally by selection.
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

        // (b) Simulate all creatures in parallel for sim_steps ticks.
        for _ in 0..cfg.sim_steps {
            world.step_all();
        }

        // (c) Collect fitnesses; preserve NEG_INFINITY for dummies.
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

        // (d) Print stats.
        let stats = GenerationStats::compute(gen as u32, &fitnesses, elapsed_ms);
        stats.print();

        if gen == 0 {
            baseline_best = stats.best_fitness;
        }
        if gen > 0 && gen % 50 == 0 {
            stats.print_milestone(baseline_best);
        }

        // (e) Save champion genome.
        if let Some(best_idx) = best_index(&fitnesses) {
            save_champion(
                &population[best_idx],
                gen as u32,
                fitnesses[best_idx],
                "champions",
            );
        }

        // (f) Evolve next generation.
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

/// Returns the index of the highest finite fitness in `fitnesses`, or `None`
/// if the slice is empty.
fn best_index(fitnesses: &[f32]) -> Option<usize> {
    fitnesses
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(i, _)| i)
}

/// Creates a minimal two-particle creature with `fitness = NEG_INFINITY` to
/// stand in for a genome that failed to build.
fn make_dummy() -> Creature {
    let mut c = Creature::new(
        vec![Particle::new(0.0, 1.0, 1.0), Particle::new(1.0, 1.0, 1.0)],
        vec![Muscle::bone(0, 1, 1.0)],
    );
    c.fitness = f32::NEG_INFINITY;
    c
}
